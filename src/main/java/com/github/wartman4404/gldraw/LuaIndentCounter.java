package com.github.wartman4404.gldraw;

import java.util.regex.Matcher;
import java.util.regex.Pattern;
import com.cyanogenmod.filemanager.ash.indent.LineIndentCounter;

class LuaIndentCounter extends LineIndentCounter {
   private static final Pattern luaRegex = Pattern.compile("(\\b(?:function|if|for|while|repeat|else|elseif|do|then)\\b|\\(|\\{)|(\\b(?:end|else|until)\\b|\\}|]])");
   private Matcher matcher;
   public LuaIndentCounter() {
      matcher = luaRegex.matcher("");
   }
   @Override public void getLineIndents(CharSequence dest, int dstart) {
      int pos;
      int linecount = 0;
      int start;
      int end = dstart - 1;

      pos = end;
      while (pos >= 0 && dest.charAt(pos) != '\n') {
         pos -= 1;
      }
      pos += 1;
      start = pos;

      Matcher m = this.matcher;
      m.reset(dest);
      m.region(start, end + 1);
      while (m.find()) {
         int add = m.group(1).isEmpty() ? -1 : 1;
         linecount += add;
      }
      this.pos = start;
      this.linecount = linecount;
   }
}
