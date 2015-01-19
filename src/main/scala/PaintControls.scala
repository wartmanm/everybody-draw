package com.github.wartman4404.gldraw

import android.os.Bundle
import android.content.Context
import android.view.View
import android.widget.{AdapterView, Adapter, GridView, ListAdapter}
import android.util.Log

import java.io.{InputStream, OutputStream, OutputStreamWriter, BufferedWriter}

import unibrush.UniBrush

import spray.json._
import PaintControls._
import GLResultTypeDef._

class PaintControls
  (val animpicker: UP[CopyShader], val brushpicker: UP[Texture], val paintpicker: UP[PointShader], val interppicker: UP[LuaScript], val unipicker: UP[UniBrush], val copypicker: UUP[CopyShader], val sidebar: FIP) 
extends AutoProductFormat {

  val namedPickers = Map(
    "anim" -> animpicker,
    "brush" -> brushpicker,
    "paint" -> paintpicker,
    "interp" -> interppicker,
    "unibrush" -> unipicker,
    "copy" -> copypicker,
    "sidebar" -> sidebar
  )

  def restoreState() = namedPickers.values.foreach(_.restoreState())
  def updateState() = namedPickers.values.foreach(_.updateState())
  def saveToString(): String = {
    namedPickers.map({case (k,v) => (k, v.save())}).toJson.toString
  }
  def loadFromString(s: String) = {
    val saved = s.parseJson.convertTo[Map[String, JsValue]]
    for ((name, state) <- saved) {
      namedPickers(name).load(state)
    }
  }
  def save(b: Bundle): Unit = b.putString("paintcontrols", saveToString())
  def load(b: Bundle): Unit = loadFromString(b.getString("paintcontrols"))
  def save(os: OutputStream): Unit = {
    val writer = new BufferedWriter(new OutputStreamWriter(os))
    writer.write(saveToString())
    writer.close()
  }
  def load(is: InputStream): Unit = {
    val state = DrawFiles.readStream(is)
    loadFromString(state)
  }
}
object PaintControls extends AndroidImplicits {
  type LAV = AdapterView[ListAdapter]
  type UP[T] = UnnamedPicker[T]
  type UUP[T] = UnnamedUnpicker[T]
  type FIP = FixedIndexPicker
  def apply
  (animpicker: LAV, brushpicker: LAV, paintpicker: LAV, interppicker: LAV, unipicker: LAV, sidebar: LAV) = {
    new PaintControls (
      new UnnamedPicker[CopyShader](animpicker),
      new UnnamedPicker[Texture](brushpicker),
      new UnnamedPicker[PointShader](paintpicker),
      new UnnamedPicker[LuaScript](interppicker),
      new UnnamedPicker[UniBrush](unipicker),
      new UnnamedUnpicker[CopyShader](None),
      new FixedIndexPicker(sidebar))
  }

  trait SavedControl {
    def save(): JsValue
    def load(j: JsValue): Unit
    def restoreState() { }
    def updateState() { }
  }

  trait SelectedListener {
    val control: AdapterView[ListAdapter]
    var selected = AdapterView.INVALID_POSITION
    def setListener(cb: (View, Int) => Unit) = {
      control.setOnItemClickListener((v: View, pos: Int) => {
        selected = pos
        cb(v, pos)
      })
    }
  }

  trait GLControl[T] {
    var enabled: Boolean = true
    def currentValue(gl: GLInit): GLResult[T]
  }

  class UnnamedPicker[T](override val control: AdapterView[ListAdapter]) extends SavedControl with GLControl[T] with SelectedListener with AutoProductFormat {
    type U = AdapterView[LazyPicker[T]]
    override def currentValue(gl: GLInit) = {
      Log.i("picker", s"getting value at idx ${selected}: '${adapter.lazified(selected)._1}'")
      adapter.getState(selected, gl)
    }
    var selectedName = ""
    private var adapter: LazyPicker[T] = null
    def setAdapter(a: LazyPicker[T]) = {
      adapter = a
      control.setAdapter(a)
    }
    override def restoreState(): Unit = {
      Log.i("picker", "restoring unnamedpicker state")
      selected = this.adapter.lazified.indexWhere(_._1 == selectedName) match {
        case -1 => 0
        case  x => x
      }
      if (enabled) this.control.performItemClick(null, selected, selected)
    }
    override def updateState() = selectedName = selected match {
      case AdapterView.INVALID_POSITION => ""
      case x => adapter.lazified(x)._1
    }
    override def save() = SavedState(enabled, selectedName).toJson
    override def load(j: JsValue) = {
      val state = j.convertTo[SavedState]
      enabled = state.enabled
      selectedName = state.selectedName
    }
  }

  class FixedIndexPicker(override val control: AdapterView[ListAdapter]) extends SavedControl with SelectedListener with AutoProductFormat {
    override def restoreState(): Unit = {
      Log.i("picker", s"clicking ${selected} in sidebar")
      this.control.performItemClick(null, selected, selected)
    }
    override def updateState() = { }
    override def save() = selected.toJson
    override def load(j: JsValue) = { selected = j.convertTo[Int] }
  }

  class UnnamedUnpicker[T](var value: Option[T] = None) extends SavedControl with GLControl[T] with AutoProductFormat {
    override def save() = enabled.toJson
    override def load(j: JsValue) = enabled = j.convertTo[Boolean]
    override def currentValue(gl: GLInit) = {
      Log.i("picker", "getting unpicker value")
      value match {
        case None => Left("No value present?")
        case Some(x) => Right(x)
      }
    }
  }

  case class SavedState(enabled: Boolean, selectedName: String)

  implicit class AdapterSeq(a: Adapter) extends IndexedSeq[Object] {
    def length = a.getCount()
    def apply(pos: Int) = a.getItem(pos)
  }
}

//class SavedGridView[T](c: Context, attrs: AttributeSet, defStyleAttr: Int, defStyleRes: Int)
//extends PaintControls.GridView(c, attrs, defStyleAttr, defStyleRes) 
//with UnnamedPicker[T] {
  //def this(c: Context, attrs: AttributeSet, defStyleAttr: Int) = this(c, attrs, defStyleAttr, 0)
  //def this(c: Context, attrs: AttributeSet) = this(c, attrs, defStyleAttr, 0, 0)
  //def this(c: Context) = this(c, attrs, null, 0, 0)
//}
