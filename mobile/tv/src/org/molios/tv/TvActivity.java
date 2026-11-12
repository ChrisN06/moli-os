package org.molios.tv;

import android.app.Activity;
import android.net.Uri;
import android.os.Bundle;
import android.view.KeyEvent;
import android.view.WindowInsets;
import android.view.WindowInsetsController;
import android.webkit.WebResourceRequest;
import android.webkit.WebResourceResponse;
import android.webkit.WebView;
import android.webkit.WebViewClient;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.util.Collections;

public final class TvActivity extends Activity {
    private WebView dashboard;

    @Override
    public void onCreate(Bundle state) {
        super.onCreate(state);
        Uri server;
        try (BufferedReader config = new BufferedReader(new InputStreamReader(getAssets().open("server.txt"), StandardCharsets.UTF_8))) {
            server = Uri.parse(config.readLine());
        } catch (IOException error) {
            throw new IllegalStateException("Missing Moli server configuration", error);
        }

        dashboard = new WebView(this);
        dashboard.getSettings().setJavaScriptEnabled(true);
        dashboard.getSettings().setDomStorageEnabled(true);
        dashboard.getSettings().setMediaPlaybackRequiresUserGesture(false);
        dashboard.getSettings().setAllowFileAccess(false);
        dashboard.getSettings().setAllowContentAccess(false);
        dashboard.getSettings().setUseWideViewPort(true);
        dashboard.setWebViewClient(new WebViewClient() {
            @Override
            public boolean shouldOverrideUrlLoading(WebView view, WebResourceRequest request) {
                return !sameServer(server, request.getUrl());
            }

            @Override
            public WebResourceResponse shouldInterceptRequest(WebView view, WebResourceRequest request) {
                Uri url = request.getUrl();
                if (!sameServer(server, url) || !request.getMethod().equals("GET")) return null;
                String path = url.getPath();
                if ("/".equals(path)) return asset("index.html", "text/html");
                if (path == null || !path.matches("/assets/[a-zA-Z0-9_.-]+")) return null;
                String type = path.endsWith(".css") ? "text/css"
                    : path.endsWith(".woff2") ? "font/woff2" : "text/javascript";
                return asset(path.substring(1), type);
            }
        });
        setContentView(dashboard);
        getWindow().getInsetsController().hide(WindowInsets.Type.systemBars());
        dashboard.requestFocus();
        if (state == null || dashboard.restoreState(state) == null) {
            dashboard.loadUrl(server + "/?tv=1#/");
        }
    }

    private static boolean sameServer(Uri server, Uri url) {
        return server.getScheme().equals(url.getScheme()) && server.getAuthority().equals(url.getAuthority());
    }

    private WebResourceResponse asset(String path, String mime) {
        try {
            return new WebResourceResponse(mime, mime.equals("font/woff2") ? null : "UTF-8",
                200, "OK", Collections.singletonMap("Cache-Control", "no-store"), getAssets().open("web/" + path));
        } catch (IOException error) {
            return new WebResourceResponse("text/plain", "UTF-8", 404, "Not Found", Collections.emptyMap(), null);
        }
    }

    @Override
    public boolean dispatchKeyEvent(KeyEvent event) {
        if (event.getKeyCode() == KeyEvent.KEYCODE_DPAD_CENTER) {
            return super.dispatchKeyEvent(new KeyEvent(event.getDownTime(), event.getEventTime(),
                event.getAction(), KeyEvent.KEYCODE_ENTER, event.getRepeatCount(), event.getMetaState(),
                event.getDeviceId(), event.getScanCode(), event.getFlags(), event.getSource()));
        }
        return super.dispatchKeyEvent(event);
    }

    @Override
    public void onBackPressed() {
        dashboard.evaluateJavascript(
            "(() => { const modal = [...document.querySelectorAll('dialog:modal')].at(-1); " +
            "if (!modal) return false; " +
            "if (modal.requestClose) modal.requestClose(); else modal.close(); return true; })()",
            closed -> {
                if ("true".equals(closed)) return;
                if (dashboard.canGoBack()) dashboard.goBack();
                else finish();
            });
    }

    @Override
    protected void onSaveInstanceState(Bundle state) {
        dashboard.saveState(state);
        super.onSaveInstanceState(state);
    }
}
