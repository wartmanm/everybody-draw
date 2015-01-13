package com.github.wartman4404.gldraw;
import android.text.InputFilter;
import android.text.Spanned;
import com.cyanogenmod.filemanager.ash.indent.LineIndentCounter;

class IndentLineFilter implements InputFilter {
    private LineIndentCounter counter;
    private StringBuilder replacer;
    public IndentLineFilter(LineIndentCounter counter) {
        this.counter = counter;
        this.replacer = new StringBuilder();
    }
    public void setIndentCounter(LineIndentCounter counter) {
        this.counter = counter;
    }
    @Override public CharSequence filter (CharSequence source, int start, int end, Spanned dest, int dstart, int dend) {
        int indentCount = 0;
        if (!source.subSequence(start, end).toString().contains("\n")) {
            return null;
        }
        int tabCount = 0;
        if (dest.length() > 0) {
            counter.getLineIndents(dest, dstart);
            int linecount = counter.linecount;
            int pos = counter.pos + 1;
            for (; pos < dstart && Character.isWhitespace(dest.charAt(pos)); pos++) tabCount++;
            if (linecount < 0) tabCount -= 2;
            else if (linecount > 0) tabCount += 2;
        }
        replacer.setLength(0);
        replacer.append('\n');
        for (int i = 0; i < tabCount; i++) replacer.append(' ');
        String result = source.subSequence(start, end).toString().replace("\n", replacer.toString());
        return result;
    }
}
