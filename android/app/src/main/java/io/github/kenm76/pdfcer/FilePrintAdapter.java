package io.github.kenm76.pdfcer;

import android.app.Activity;
import android.os.Bundle;
import android.os.CancellationSignal;
import android.os.ParcelFileDescriptor;
import android.print.PageRange;
import android.print.PrintAttributes;
import android.print.PrintDocumentAdapter;
import android.print.PrintDocumentInfo;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;

/**
 * Gives Android's print service one finished PDF. The service lays the pages
 * out for the chosen paper and selects the page range itself, so every write
 * delivers the whole file. The file is a private copy for this job and is
 * deleted when the print dialog closes.
 */
final class FilePrintAdapter extends PrintDocumentAdapter {
    private final Activity activity;
    private final File file;
    private final String name;

    FilePrintAdapter(Activity activity, File file, String name) {
        this.activity = activity;
        this.file = file;
        this.name = name;
    }

    @Override
    public void onLayout(
            PrintAttributes oldAttributes,
            PrintAttributes newAttributes,
            CancellationSignal cancel,
            LayoutResultCallback callback,
            Bundle extras) {
        if (cancel.isCanceled()) {
            callback.onLayoutCancelled();
            return;
        }
        PrintDocumentInfo info =
                new PrintDocumentInfo.Builder(name)
                        .setContentType(PrintDocumentInfo.CONTENT_TYPE_DOCUMENT)
                        .build();
        callback.onLayoutFinished(info, false);
    }

    /** Copies on a thread of its own: this method is called on the main thread. */
    @Override
    public void onWrite(
            PageRange[] pages,
            ParcelFileDescriptor destination,
            CancellationSignal cancel,
            WriteResultCallback callback) {
        new Thread(() -> {
            String failure = copyTo(destination, cancel);
            activity.runOnUiThread(() -> {
                if (cancel.isCanceled()) {
                    callback.onWriteCancelled();
                } else if (failure != null) {
                    callback.onWriteFailed(failure);
                } else {
                    callback.onWriteFinished(new PageRange[] {PageRange.ALL_PAGES});
                }
            });
        }).start();
    }

    /** The reason the copy failed, or null. */
    private String copyTo(ParcelFileDescriptor destination, CancellationSignal cancel) {
        try (InputStream in = new FileInputStream(file);
             OutputStream out = new FileOutputStream(destination.getFileDescriptor())) {
            byte[] buf = new byte[1 << 16];
            for (int n; !cancel.isCanceled() && (n = in.read(buf)) > 0; ) out.write(buf, 0, n);
            return null;
        } catch (IOException e) {
            return e.getMessage();
        }
    }

    @Override
    public void onFinish() {
        file.delete();
    }
}
