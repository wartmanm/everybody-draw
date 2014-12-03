package com.github.wartman4404.gldraw

import android.widget._
import android.view._
import android.os.Bundle
import android.content.DialogInterface
import java.lang.Runnable
import scala.collection.mutable.ArrayBuffer
import android.util.{JsonReader, JsonWriter}

trait AndroidImplicits {
  //dialog
  //trait DialogCallback {
		//def returnFromSetup(result: Bundle)
		//def startDialog()
		//def stopDialog()
  //}

  implicit def makeDialogOnClickListener(fn: ()=>Any):DialogInterface.OnClickListener = new DialogInterface.OnClickListener() {
	  override def onClick(dialog: DialogInterface, which: Int) = fn()
  }

  // views
  implicit def makeAnyViewOnClickListener(fn: ()=>Any):View.OnClickListener = new View.OnClickListener() {
    override def onClick(view: View) = fn()
  }
  implicit def makeViewOnClickListener(fn: (View)=>Any):View.OnClickListener = new View.OnClickListener() {
    override def onClick(view: View) = fn(view)
  }
  implicit def makeViewOnLongClickListener(fn: ()=>Boolean):View.OnLongClickListener = new View.OnLongClickListener() {
    override def onLongClick(view: View) = fn()
  }
  implicit def makeListOnClickListener(fn: (View, Int)=>Any) = new AdapterView.OnItemClickListener() {
    override def onItemClick(parent: AdapterView[_], view: View, pos: Int, id: Long) = fn(view, pos)
  }
  implicit def makeListOnLongClickListener(fn: (View, Int)=>Boolean) = new AdapterView.OnItemLongClickListener() {
    override def onItemLongClick(parent: AdapterView[_], view: View, pos: Int, id: Long) = fn(view, pos)
  }
  implicit def makeOnTouchListener(fn: (MotionEvent)=>Boolean) = new View.OnTouchListener() {
    override def onTouch(v: View, event: MotionEvent) = fn(event)
  }
  implicit def makeRunnable(fn: ()=>Unit) = new Runnable() {
    override def run() = fn()
  }

  // iterators
  implicit def viewGroupAsSeq(vg: ViewGroup) = new IndexedSeq[View] {
    def apply(i: Int) = vg.getChildAt(i)
    def length = vg.getChildCount()
  }

  implicit class JsonReaderHelper(j: JsonReader) {
    def readObject[T](cb: (String) => Unit): Unit = {
      j.beginObject()
        while (j.hasNext()) cb(j.nextName())
      j.endObject()
    }
    def readArray[T](cb: (JsonReader) => T): Seq[T] = {
      val arr = new ArrayBuffer[T]()
      j.beginArray()
        while(j.hasNext()) arr += cb(j)
      j.endArray()
      arr
    }
  }
}
