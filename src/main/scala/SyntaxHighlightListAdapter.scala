package com.github.wartman4404.gldraw
import android.view._
import android.widget._
import android.content.Context

import SyntaxHighlightListAdapter._

class SyntaxHighlightListAdapter(context: Context, content: Array[Sources]) extends BaseAdapter {
  import TypedResource._
  val inflater = LayoutInflater.from(context)


  override def areAllItemsEnabled = true
  override def isEnabled(pos: Int) = true
  override def getCount = content.size
  override def getViewTypeCount() = 2
  override def getItem(pos: Int) = content(pos)
  override def getItemViewType(position: Int) = content(position) match {
    case _: LuaSource => 0
    case _: ShaderSource => 1
  }
  override def getItemId(position: Int) = position
  override def getView(position: Int, convertView: View, parent: ViewGroup): View = {
    val item = content(position)
    val view =
      if (convertView == null) getNewView(item, parent)
      else convertView
    setupView(view, item)
    view
  }

  private def getNewView(item: Sources, parent: ViewGroup) = item match {
    case _: LuaSource => {
      val view = inflater.inflate(R.layout.debug_lua, parent, false)
      val code = view.findView(TR.code)
      code.setSyntaxListener(LuaSyntaxHighlightProcessor.LuaProcessor)
      view.setTag(LuaHolder(view.findView(TR.title), code))
      view
    }
    case _: ShaderSource => {
      val view = inflater.inflate(R.layout.debug_shader, parent, false)
      val vert = view.findView(TR.vert)
      val frag = view.findView(TR.frag)
      vert.setSyntaxListener(GLSLSyntaxHighlightProcessor.VertProcessor)
      frag.setSyntaxListener(GLSLSyntaxHighlightProcessor.FragProcessor)
      view.setTag(ShaderHolder(view.findView(TR.title), view.findView(TR.vert), view.findView(TR.frag)))
      view
    }
  }
  private def setupView(view: View, item: Sources) = item match {
    case item: LuaSource => {
      val holder = view.getTag().asInstanceOf[LuaHolder]
      holder.title.setText(item.name)
      holder.code.setText(item.source)
    }
    case item: ShaderSource => {
      val holder = view.getTag().asInstanceOf[ShaderHolder]
      holder.title.setText(item.name)
      holder.vert.setText(item.vert)
      holder.frag.setText(item.frag)
    }
  }
}
object SyntaxHighlightListAdapter {
  trait Sources { }
  case class LuaSource(name: String, source: String) extends Sources
  case class ShaderSource(name: String, vert: String, frag: String) extends Sources

  case class LuaHolder(title: TextView, code: SyntaxHighlightEditText)
  case class ShaderHolder(title: TextView, vert: SyntaxHighlightEditText, frag: SyntaxHighlightEditText)
}
