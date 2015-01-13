package com.github.wartman4404.gldraw;

import android.text.TextWatcher;
import android.text.Editable;
import com.cyanogenmod.filemanager.ash.SyntaxHighlightProcessor;

class HighlightWatcher implements TextWatcher {
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
