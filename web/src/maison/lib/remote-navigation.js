const DIRECTIONS = {
  ArrowLeft: ['x', -1],
  ArrowRight: ['x', 1],
  ArrowUp: ['y', -1],
  ArrowDown: ['y', 1],
};

/** Return the nearest control in the requested direction, without wrapping. */
export function nearestTarget(rectangles, index, key) {
  const direction = Object.hasOwn(DIRECTIONS, key) ? DIRECTIONS[key] : null;
  const valid = (r) => r && [r.left, r.top, r.width, r.height].every(Number.isFinite) && r.width > 0 && r.height > 0;
  const from = rectangles[index];
  if (!direction || !valid(from)) return null;
  const [axis, sign] = direction;
  const horizontal = axis === 'x';
  let nearest = null;
  let score = Infinity;
  rectangles.forEach((to, candidate) => {
    if (candidate === index || !valid(to)) return;
    const dx = to.left + to.width / 2 - from.left - from.width / 2;
    const dy = to.top + to.height / 2 - from.top - from.height / 2;
    const forward = (horizontal ? dx : dy) * sign;
    if (forward <= 1) return;
    const sideways = Math.abs(horizontal ? dy : dx);
    const distance = forward + sideways * 2.5;
    if (distance < score) {
      score = distance;
      nearest = candidate;
    }
  });
  return nearest;
}

/** Add arrow navigation while native editors and existing key handlers keep ownership. */
export function attachRemoteNavigation(root, { television = false } = {}) {
  const document = root.ownerDocument;
  const listener = television ? document : root;
  const content = television ? root.querySelector('main') : null;
  const visible = (element) => {
    if (!element.getClientRects().length || element.closest('[hidden], [inert], [aria-hidden="true"]')) return false;
    const style = document.defaultView.getComputedStyle(element);
    return style.visibility !== 'hidden' && style.visibility !== 'collapse' && style.display !== 'none';
  };
  const keydown = (event) => {
    if (!Object.hasOwn(DIRECTIONS, event.key) || event.defaultPrevented || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
    const target = event.target;
    if (television && target !== document.body && !root.contains(target)) return;
    const editor = target.closest('input, textarea, select, [role="slider"], [contenteditable]:not([contenteditable="false"])');
    if (editor) {
      const horizontal = event.key === 'ArrowLeft' || event.key === 'ArrowRight';
      if (editor.matches('input[type="range"], [role="slider"]')) {
        if (horizontal) return;
      } else if (editor.matches('select')) {
        if (!horizontal) return;
      } else return;
    }
    const dialogs = [...root.querySelectorAll('dialog[open], [role="dialog"]')].filter(visible);
    const scope = dialogs.at(-1) ?? root;
    const controls = [...scope.querySelectorAll('button, a[href], input, select, textarea, [tabindex]')]
      .filter((element) => element.tabIndex >= 0 && !element.matches(':disabled') && !element.closest('[aria-disabled="true"]') && visible(element));
    const current = controls.indexOf(document.activeElement);
    const next = current < 0 ? 0 : nearestTarget(controls.map((element) => element.getBoundingClientRect()), current, event.key);
    const destination = controls[next];
    // A final block of text or a chart can extend below the last focusable control.
    const vertical = DIRECTIONS[event.key][0] === 'y';
    const scrollport = television ? (scope === root ? content : scope) : null;
    if (vertical && scrollport?.contains(controls[current] ?? null) && !scrollport.contains(destination ?? null)) {
      const direction = DIRECTIONS[event.key][1];
      const remaining = direction > 0 ? scrollport.scrollHeight - scrollport.clientHeight - scrollport.scrollTop : scrollport.scrollTop;
      if (remaining > 1) {
        event.preventDefault();
        scrollport.scrollBy({ top: direction * Math.min(remaining, scrollport.clientHeight * .6), behavior: 'instant' });
        return;
      }
    }
    if (!controls.length) return;
    event.preventDefault();
    if (!destination) return;
    destination.focus({ preventScroll: true });
    destination.scrollIntoView({ block: 'nearest', inline: 'nearest' });
  };
  listener.addEventListener('keydown', keydown);
  return () => listener.removeEventListener('keydown', keydown);
}
