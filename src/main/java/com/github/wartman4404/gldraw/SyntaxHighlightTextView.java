package com.github.wartman4404.gldraw;

import android.text.Editable;
import android.text.TextWatcher;
import android.text.InputFilter;
import android.util.AttributeSet;
import android.content.Context;
import android.widget.EditText;
import android.widget.TextView;
import com.cyanogenmod.filemanager.ash.SyntaxHighlightProcessor;
import com.cyanogenmod.filemanager.ash.indent.LineIndentCounter;
import com.cyanogenmod.filemanager.ash.indent.BracketIndentCounter;

public class SyntaxHighlightTextView extends EditText {
    private TextWatcher syntaxHighlightListener;
    private IndentLineFilter newlineFilter;

    public void refresh() { }

    public SyntaxHighlightTextView(Context context) {
        super(context);
        init();
    }

    public SyntaxHighlightTextView(Context context, AttributeSet attrs) {
        super(context, attrs);
        init();
    }

    public SyntaxHighlightTextView(Context context, AttributeSet attrs, int defStyle) {
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
}
