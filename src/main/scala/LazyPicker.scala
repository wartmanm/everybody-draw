package com.github.wartman4404.gldraw
import android.view._
import android.widget._
import android.content.Context
import GLResultTypeDef._

class LazyPicker[T](context: Context, thread: TextureSurfaceThread, content: Seq[(String, (GLInit)=>GLResult[T])]) extends BaseAdapter {
  val inflater = LayoutInflater.from(context)
  val lazified: Seq[(String, LoadedState[T])] = content.map { case (k, v) => (k, new LoadedState(v)) }
  case class Holder(nameView: TextView)

  override def areAllItemsEnabled = false
  override def isEnabled(pos: Int) = lazified(pos)._2.isNotFailed
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
      view = inflater.inflate(android.R.layout.simple_list_item_activated_1, parent, false)
      val text = view.findViewById(android.R.id.text1).asInstanceOf[TextView]
      holder = Holder(text)
      view.setTag(holder)
    } else {
      holder = view.getTag().asInstanceOf[Holder]
    }
    val nameview = holder.nameView
    nameview.setText(item._1)
    val ok = item._2.isNotFailed
    nameview.setEnabled(ok)
    view.setEnabled(ok)
    //view.setBackgroundColor(if ok 0xffff0000 else 0x00000000);
    view
  }

  def getState(pos: Int, gl: GLInit) = lazified(pos)._2.get(gl)

  class LoadedState[T](var loader: (GLInit)=>GLResult[T]) {
    def get(gl: GLInit): GLResult[T] = {
      cachedValue match {
        case None => {
          val value = loader(gl)
          cachedValue = Some(value)
          value
        }
        case Some(value) => value
      }
    }
    private var cachedValue: Option[GLResult[T]] = None

    def isNotFailed = cachedValue match {
      case None => true
      case Some(x) => x.isRight
    }
  }
}
