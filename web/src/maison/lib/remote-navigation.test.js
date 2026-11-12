import test from 'node:test';
import assert from 'node:assert/strict';
import { nearestTarget, attachRemoteNavigation } from './remote-navigation.js';

const rect = (left, top, width = 80, height = 40) => ({ left, top, width, height });

test('arrows follow the displayed grid in all four directions', () => {
  const grid = [rect(0, 0), rect(100, 0), rect(0, 80), rect(100, 80)];
  assert.equal(nearestTarget(grid, 0, 'ArrowRight'), 1);
  assert.equal(nearestTarget(grid, 0, 'ArrowDown'), 2);
  assert.equal(nearestTarget(grid, 3, 'ArrowLeft'), 2);
  assert.equal(nearestTarget(grid, 3, 'ArrowUp'), 1);
});

test('boundaries never wrap to an unrelated row or opposite direction', () => {
  const grid = [rect(0, 0), rect(100, 0), rect(0, 80)];
  assert.equal(nearestTarget(grid, 0, 'ArrowLeft'), null);
  assert.equal(nearestTarget(grid, 0, 'ArrowUp'), null);
  assert.equal(nearestTarget(grid, 1, 'ArrowRight'), null);
});

test('aligned controls win over a nearby diagonal control', () => {
  const grid = [rect(0, 0), rect(50, 70), rect(150, 0)];
  assert.equal(nearestTarget(grid, 0, 'ArrowRight'), 2);
});

test('geometry uses rectangle centers, including differently sized controls', () => {
  const grid = [rect(100, 0, 200), rect(0, 0, 60), rect(310, 0, 20)];
  assert.equal(nearestTarget(grid, 0, 'ArrowLeft'), 1);
  assert.equal(nearestTarget(grid, 0, 'ArrowRight'), 2);
});

test('empty, invalid, zero-sized geometry and unsupported keys have no target', () => {
  assert.equal(nearestTarget([], 0, 'ArrowRight'), null);
  assert.equal(nearestTarget([rect(0, 0)], -1, 'ArrowRight'), null);
  assert.equal(nearestTarget([null, rect(100, 0)], 0, 'ArrowRight'), null);
  assert.equal(nearestTarget([rect(0, 0), rect(NaN, 0), rect(100, 0, 0), null], 0, 'ArrowRight'), null);
  assert.equal(nearestTarget([rect(0, 0), rect(100, 0)], 0, 'Enter'), null);
  assert.equal(nearestTarget([rect(0, 0), rect(100, 0)], 0, 'constructor'), null);
});

function fixture() {
  const document = { activeElement: null, defaultView: { getComputedStyle: () => ({ visibility: 'visible', display: 'block' }) } };
  const control = (left, options = {}) => ({
    tabIndex: 0,
    getClientRects: () => options.hidden ? [] : [rect(left, 0)],
    getBoundingClientRect: () => rect(left, 0),
    kind: options.kind,
    matches(selector) {
      if (selector === ':disabled') return !!options.disabled;
      if (selector === 'select') return this.kind === 'select';
      return selector.startsWith('input[type="range"]') && ['range', 'slider'].includes(this.kind);
    },
    closest(selector) { return selector.startsWith('input') && (options.editor || this.kind) ? this : null; },
    focus() { document.activeElement = this; },
    scrollIntoView() { this.scrolled = true; },
  });
  const first = control(0), second = control(100), disabled = control(80, { disabled: true }), hidden = control(90, { hidden: true });
  let listener;
  let dialogs = [];
  const root = {
    ownerDocument: document,
    querySelectorAll: (selector) => selector.startsWith('dialog') ? dialogs : [first, disabled, hidden, second],
    addEventListener: (_, handler) => { listener = handler; },
    removeEventListener: (_, handler) => { assert.equal(handler, listener); listener = null; },
  };
  const event = (target = first, key = 'ArrowRight') => ({ target, key, prevented: false, preventDefault() { this.prevented = true; } });
  return { document, root, first, second, control, event, setDialogs: (value) => { dialogs = value; }, dispatch: (value) => listener?.(value), get listener() { return listener; } };
}

test('attachment has no focus effect, filters controls and returns cleanup', () => {
  const f = fixture(), cleanup = attachRemoteNavigation(f.root);
  assert.equal(f.document.activeElement, null);
  f.document.activeElement = f.first;
  const event = f.event();
  f.dispatch(event);
  assert.equal(f.document.activeElement, f.second);
  assert.equal(f.second.scrolled, true);
  assert.equal(event.prevented, true);
  cleanup();
  assert.equal(f.listener, null);
});

test('native editors, handled keys and browser shortcuts remain untouched', () => {
  const f = fixture();
  attachRemoteNavigation(f.root);
  f.document.activeElement = f.first;
  for (const event of [f.event(f.control(0, { editor: true })), { ...f.event(), defaultPrevented: true }, { ...f.event(), altKey: true }, f.event(f.first, 'Escape'), f.event(f.first, 'Enter')]) {
    f.dispatch(event);
    assert.equal(event.prevented, false);
    assert.equal(f.document.activeElement, f.first);
  }
});

test('arrow navigation stays inside the last visible dialog', () => {
  const f = fixture(), modalFirst = f.control(500), modalSecond = f.control(600);
  const dialog = { getClientRects: () => [rect(500, 0)], closest: () => null, querySelectorAll: () => [modalFirst, modalSecond] };
  f.setDialogs([dialog]);
  attachRemoteNavigation(f.root);
  f.document.activeElement = f.first;
  f.dispatch(f.event());
  assert.equal(f.document.activeElement, modalFirst);
  f.dispatch(f.event(modalFirst));
  assert.equal(f.document.activeElement, modalSecond);
  f.dispatch(f.event(modalSecond));
  assert.equal(f.document.activeElement, modalSecond);
});


test('ranges and sliders keep value arrows but vertical arrows can leave', () => {
  for (const kind of ['range', 'slider']) {
    const f = fixture();
    f.first.kind = kind;
    f.second.getBoundingClientRect = () => rect(0, 80);
    attachRemoteNavigation(f.root);
    f.document.activeElement = f.first;
    const valueArrow = f.event(f.first, 'ArrowRight');
    f.dispatch(valueArrow);
    assert.equal(valueArrow.prevented, false);
    assert.equal(f.document.activeElement, f.first);
    const leaveArrow = f.event(f.first, 'ArrowDown');
    f.dispatch(leaveArrow);
    assert.equal(leaveArrow.prevented, true);
    assert.equal(f.document.activeElement, f.second);
  }
});

test('selects keep option arrows but horizontal arrows can leave', () => {
  const f = fixture();
  f.first.kind = 'select';
  attachRemoteNavigation(f.root);
  f.document.activeElement = f.first;
  const optionArrow = f.event(f.first, 'ArrowDown');
  f.dispatch(optionArrow);
  assert.equal(optionArrow.prevented, false);
  assert.equal(f.document.activeElement, f.first);
  const leaveArrow = f.event(f.first, 'ArrowRight');
  f.dispatch(leaveArrow);
  assert.equal(leaveArrow.prevented, true);
  assert.equal(f.document.activeElement, f.second);
});

test('TV arrows reveal text below the last control before moving to the dock', () => {
  const f = fixture();
  f.document.addEventListener = f.root.addEventListener;
  f.document.removeEventListener = f.root.removeEventListener;
  f.root.contains = () => true;
  const content = {
    contains: (control) => control === f.first,
    clientHeight: 100,
    scrollHeight: 170,
    scrollTop: 0,
    scrollBy({ top }) { this.scrollTop += top; },
  };
  f.root.querySelector = () => content;
  f.second.getBoundingClientRect = () => rect(0, 120);
  attachRemoteNavigation(f.root, { television: true });
  f.document.activeElement = f.first;
  f.dispatch(f.event(f.first, 'ArrowDown'));
  f.dispatch(f.event(f.first, 'ArrowDown'));
  assert.equal(content.scrollTop, 70);
  assert.equal(f.document.activeElement, f.first);
  f.dispatch(f.event(f.first, 'ArrowDown'));
  assert.equal(f.document.activeElement, f.second);
});
