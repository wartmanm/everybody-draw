package com.github.wartman4404.gldraw;

import android.text.*;
import android.text.style.*;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import com.cyanogenmod.filemanager.ash.indent.LineIndentCounter;
import com.cyanogenmod.filemanager.ash.SyntaxHighlightProcessor;

class RegexSyntaxHighlightProcessor extends SyntaxHighlightProcessor {
   private OnHighlightUpdatedListener listener;
   private final Matcher matcher;
   private final int badMatchColor;
   private final int[] colors;
   private final LineIndentCounter indentCounter;

   public RegexSyntaxHighlightProcessor(RegexValues values) {
      this.matcher = Pattern.compile(values.regex).matcher("");
      this.colors = new int[values.colors.length];
      System.arraycopy(values.colors, 0, this.colors, 0, values.colors.length);
      this.badMatchColor = values.badMatchColor;
      this.indentCounter = values.indentCounter;
   }

   public void setListener(OnHighlightUpdatedListener listener) {
      this.listener = listener;
   }

   @Override public void initialize() { }
   @Override public void process(Spannable spannable) {
      processSubsequence(spannable, 0, spannable.length());
   }

   @Override public void process(Spannable spannable, int start, int end) {
      processSubsequence(spannable, start, end);
   }

   private void processSubsequence(Spannable parent, int start, int end) {
      ForegroundColorSpan[] spans = parent.getSpans(start, end, ForegroundColorSpan.class);
      // expand to reprocess any spans that cover the selected range
      for (int i = 0; i < spans.length; i++) {
         ForegroundColorSpan span = spans[i];
         final int cstart = parent.getSpanStart(span);
         final int cend = parent.getSpanEnd(span);
         start = (start < cstart) ? start : cstart;
         end = (end > cend) ? end : cend;
         parent.removeSpan(span);
      }
      final int regionStart = (start == 0 && end == parent.length())
         ? 0
         : start;
      Matcher m = matcher;
      m.reset(parent);
      m.region(regionStart, parent.length());
      while (m.find()) {
         final int mstart = m.start();
         final int mend = m.end();
         if (mstart >= end) {
            if (this.listener != null) {
               this.listener.onHighlightUpdated(parent, start, end);
            }
            return;
         }
         if (mend > end) {
            // this match extends past the cleared range; clear it and expand again
            spans = parent.getSpans(end, mend, ForegroundColorSpan.class);
            end = mend;
            for (int i = 0; i < spans.length; i++) {
               ForegroundColorSpan span = spans[i];
               final int cend = parent.getSpanEnd(span);
               end = (end > cend) ? end : cend;
               parent.removeSpan(span);
            }
         }
         ForegroundColorSpan span = new ForegroundColorSpan(getMatchColor(m));
         parent.setSpan(span, mstart, mend, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE);
      }
      if (this.listener != null) {
         this.listener.onHighlightUpdated(parent, start, end);
      }
   }

   private int getMatchColor(Matcher m) {
      for (int i = 1; i <= colors.length; i++) {
         if (m.group(i) != null) {
            return colors[i-1];
         }
      }
      return this.badMatchColor;
   }

   @Override public void cancel() { }

   @Override public LineIndentCounter getIndentCounter() {
      return this.indentCounter;
   }

   public static interface OnHighlightUpdatedListener {
      void onHighlightUpdated(Spannable spannable, int start, int end);
   }
   
   public static class RegexValues {
      public String regex;
      public int[] colors;
      public int badMatchColor;
      public LineIndentCounter indentCounter;

      public RegexValues(String regex, LineIndentCounter indentCounter, int[] colors) {
         this(regex, indentCounter, colors, 0xffff0000);
      }
      public RegexValues(String regex, LineIndentCounter indentCounter, int[] colors,
            int badMatchColor) {
         this.regex = regex;
         this.colors = colors;
         this.badMatchColor = badMatchColor;
         this.indentCounter = indentCounter;
      }
   }
}
