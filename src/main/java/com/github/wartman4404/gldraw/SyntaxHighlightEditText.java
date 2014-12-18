package com.github.wartman4404.gldraw;

import android.view.*;
import android.text.*;
import android.util.AttributeSet;
import android.content.Context;
import android.widget.EditText;
import android.widget.TextView;
import com.cyanogenmod.filemanager.ash.SyntaxHighlightProcessor;
import com.cyanogenmod.filemanager.ash.indent.LineIndentCounter;
import com.cyanogenmod.filemanager.ash.indent.BracketIndentCounter;

import java.util.regex.Matcher;
import java.util.regex.Pattern;


public class SyntaxHighlightEditText extends EditText {
    private TextWatcher syntaxHighlightListener;
    private IndentLineFilter newlineFilter;

    public void refresh() { }

    public SyntaxHighlightEditText(Context context) {
        super(context);
        init();
    }

    public SyntaxHighlightEditText(Context context, AttributeSet attrs) {
        super(context, attrs);
        init();
    }

    public SyntaxHighlightEditText(Context context, AttributeSet attrs, int defStyle) {
        super(context, attrs, defStyle);
        init();
    }

    private void init() {
        this.setText(this.getText(), TextView.BufferType.EDITABLE);
        Editable thisText = this.getEditableText();
        this.newlineFilter = new IndentLineFilter(new BracketIndentCounter());
        this.setFilters(new InputFilter[] { this.newlineFilter });
        this.syntaxHighlightListener = null;
    }

    public void setSyntaxListener(SyntaxHighlightProcessor processor) {
        if (syntaxHighlightListener != null) {
            this.removeTextChangedListener(syntaxHighlightListener);
        }
        syntaxHighlightListener = new HighlightWatcher(processor);
        LineIndentCounter customCounter = processor.getIndentCounter();
        if (customCounter != null) {
            this.newlineFilter.setIndentCounter(customCounter);
        }
        this.addTextChangedListener(syntaxHighlightListener);
    }

    private static class IndentLineFilter implements InputFilter {
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

    private static class HighlightWatcher implements TextWatcher {
        private SyntaxHighlightProcessor processor;
        public HighlightWatcher(SyntaxHighlightProcessor processor) {
            this.processor = processor;
        }
        @Override public void afterTextChanged (Editable s) { }
        @Override public void beforeTextChanged (CharSequence s, int start, int count, int after) { }
        @Override public void onTextChanged (CharSequence s, final int start, final int before, final int count) {
            int wordStart, wordEnd;
            // examine previous word in case user typed a space
            for (wordStart = start-1; wordStart >= 0 && !Character.isWhitespace(s.charAt(wordStart)); wordStart--);
            for (wordEnd = start + count; wordEnd < s.length() && !Character.isWhitespace(s.charAt(wordEnd)); wordEnd++);
            processor.process((Editable)s, wordStart + 1, wordEnd);
        }
    }

    private static class IndentLineSpan implements SpanWatcher {
        @Override public void onSpanAdded(Spannable text, Object what, int start, int end) { }
        @Override public void onSpanChanged(Spannable text, Object what, int ostart, int oend, int nstart, int nend) { }
        @Override public void onSpanRemoved(Spannable text, Object what, int start, int end) { }
    }
}
