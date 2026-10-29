package io.github.kenm76.pdfcer;

import android.content.ContentResolver;
import android.content.Intent;
import android.database.Cursor;
import android.net.Uri;
import android.os.Bundle;
import android.provider.OpenableColumns;

import androidx.core.graphics.Insets;
import androidx.core.view.WindowInsetsCompat;

import com.google.androidgamesdk.GameActivity;

import java.io.File;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.io.OutputStream;
import java.util.concurrent.SynchronousQueue;

/**
 * Hosts the Rust program, which lives in libpdfcer_android.so. The only Java
 * behaviour is what Android allows only an Activity to do: run the system
 * document picker, receive a document another app opens in pdfcer, and report
 * where the system bars cover the window. The Rust side calls {@link #pickDocuments} from its own
 * thread and blocks until the operator picks or cancels.
 */
public class MainActivity extends GameActivity {
    static {
        System.loadLibrary("pdfcer_android");
    }

    private static final int PICK = 1;
    private final SynchronousQueue<String> picked = new SynchronousQueue<>();

    /**
     * Shows the document picker and copies each chosen document into the
     * app's cache, because the engine reads files by path and a picked
     * document is a content URI.
     *
     * @param multiple whether several documents may be chosen
     * @param mimeTypes the accepted MIME types, one per line
     * @return the local paths, one per line; empty when cancelled
     */
    public String pickDocuments(boolean multiple, String mimeTypes) {
        runOnUiThread(() -> {
            Intent intent = new Intent(Intent.ACTION_OPEN_DOCUMENT);
            intent.addCategory(Intent.CATEGORY_OPENABLE);
            String[] types = mimeTypes.split("\n");
            intent.setType(types.length == 1 ? types[0] : "*/*");
            if (types.length > 1) intent.putExtra(Intent.EXTRA_MIME_TYPES, types);
            intent.putExtra(Intent.EXTRA_ALLOW_MULTIPLE, multiple);
            startActivityForResult(intent, PICK);
        });
        try {
            return picked.take();
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            return "";
        }
    }

    /** Queues a document copied from a VIEW intent; see crate pdfcer-android, module viewed. */
    private static native void nativeViewed(String path);

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        if (savedInstanceState == null) receive(getIntent());
    }

    @Override
    protected void onNewIntent(Intent intent) {
        super.onNewIntent(intent);
        receive(intent);
    }

    /** Copies a VIEW intent's document into the cache, off the UI thread, and hands it over. */
    private void receive(Intent intent) {
        if (intent == null || !Intent.ACTION_VIEW.equals(intent.getAction())) return;
        final Uri uri = intent.getData();
        if (uri == null) return;
        new Thread(() -> {
            String path = copyToCache(uri);
            if (path != null) nativeViewed(path);
        }).start();
    }

    /**
     * The window edges the status bar, navigation bar and display cutout cover,
     * in physical pixels, as "left top right bottom".
     */
    public String systemBarInsets() {
        Insets i = getWindowInsets(WindowInsetsCompat.Type.systemBars() | WindowInsetsCompat.Type.displayCutout());
        return i.left + " " + i.top + " " + i.right + " " + i.bottom;
    }

    @Override
    protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        super.onActivityResult(requestCode, resultCode, data);
        if (requestCode != PICK) return;
        StringBuilder paths = new StringBuilder();
        if (resultCode == RESULT_OK && data != null) {
            if (data.getClipData() != null) {
                for (int i = 0; i < data.getClipData().getItemCount(); i++) {
                    append(paths, copyToCache(data.getClipData().getItemAt(i).getUri()));
                }
            } else if (data.getData() != null) {
                append(paths, copyToCache(data.getData()));
            }
        }
        final String answer = paths.toString();
        new Thread(() -> {
            try {
                picked.put(answer);
            } catch (InterruptedException ignored) {
            }
        }).start();
    }

    private static void append(StringBuilder out, String path) {
        if (path == null) return;
        if (out.length() > 0) out.append('\n');
        out.append(path);
    }

    /** Copies a picked document to cache/open/<its name>; null on failure. */
    private String copyToCache(Uri uri) {
        ContentResolver resolver = getContentResolver();
        String name = "document.pdf";
        try (Cursor c = resolver.query(uri, new String[] {OpenableColumns.DISPLAY_NAME}, null, null, null)) {
            if (c != null && c.moveToFirst() && !c.isNull(0)) name = new File(c.getString(0)).getName();
        } catch (RuntimeException ignored) {
        }
        File dir = new File(getCacheDir(), "open");
        if (!dir.isDirectory() && !dir.mkdirs()) return null;
        File out = new File(dir, name);
        try (InputStream in = resolver.openInputStream(uri);
             OutputStream os = new FileOutputStream(out)) {
            if (in == null) return null;
            byte[] buf = new byte[1 << 16];
            for (int n; (n = in.read(buf)) > 0; ) os.write(buf, 0, n);
            return out.getAbsolutePath();
        } catch (java.io.IOException e) {
            return null;
        }
    }
}
