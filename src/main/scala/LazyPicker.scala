package com.github.wartman4404.gldraw
import android.view._
import android.widget._
import android.content.Context
import GLResultTypeDef._

class LazyPicker[T](context: Context, thread: TextureSurfaceThread, content: Seq[(String, (Unit)=>GLResult[T])]) extends BaseAdapter {
  val inflater = LayoutInflater.from(context)
  val lazified: Seq[(String, LoadedState[T])] = content.map { case (k, v) => (k, new LoadedState(v)) }
  case class Holder(nameView: TextView)

  override def getCount = lazified.size
  override def getViewTypeCount() = 1
  override def getItem(pos: Int) = lazified(pos)
  override def getItemViewType(position: Int) = 0
  override def getItemId(position: Int) = position
  override def getView(position: Int, convertView: View, parent: ViewGroup): View = {
    var view = convertView
    var holder: Holder = null.asInstanceOf[Holder]
    val item = lazified(position)
    if (view == null) {
      view = inflater.inflate(android.R.layout.simple_dropdown_item_1line, parent, false)
      val text = view.findViewById(android.R.id.text1).asInstanceOf[TextView]
      holder = Holder(text)
      view.setTag(holder)
    } else {
      holder = view.getTag().asInstanceOf[Holder]
    }
    holder.nameView.setText(item._1)
    view
  }
  def getState(pos: Int, cb: (GLResult[T])=>Any) = {
    lazified(pos)._2.get(cb)
  }
  //DANGER: runs callback on gl thread
  class LoadedState[T](var loader: (Unit)=>GLResult[T]) {
    def get(cb: (GLResult[T])=>Any) = {
      thread.runHere {
        val value = loader(())
        _value = value.right.toOption
        cb(value)
      }
    }

    private var _value: Option[T] = None
  }
}


  //abstract sealed class InitState[T]
  //case class NotLoaded[T](loader: (Unit)=>Option[T]) extends InitState(T)
  //case class Loaded[T](value: T) extends InitState[T]
  //case class 
