package com.github.wartman4404.gldraw

import android.os.Bundle
import android.content.Context
import android.view.View
import android.widget.{AdapterView, Adapter, GridView, ListAdapter}
import android.util.Log
import android.graphics.Bitmap

import java.io.{InputStream, OutputStream, OutputStreamWriter, BufferedWriter}

import unibrush.{UniBrush, UniBrushSource}

import PaintControls._
import GLResultTypeDef._
import java.io.{StringReader, StringWriter}
import android.util.{JsonReader, JsonWriter}

class PaintControls
  (val animpicker: UP[CopyShader], val brushpicker: UP[Texture], val paintpicker: UP[PointShader], val interppicker: UP[LuaScript], val unipicker: UP[UniBrush], val copypicker: UUP[CopyShader], val sidebar: FIP) {

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
    val writer = new StringWriter()
    val j = new JsonWriter(writer)
    j.beginObject()
      for ((k, v) <- namedPickers) {
        j.name(k)
        v.save(j)
      }
    j.endObject()
    writer.close()
    writer.toString()
  }
  def loadFromString(s: String) = {
    val reader = new JsonReader(new StringReader(s))
    reader.beginObject()
    while (reader.hasNext()) {
      namedPickers(reader.nextName()).load(reader)
    }
    reader.endObject()
    reader.close()
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
  type UP[U] = UnnamedPicker[U]
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
    def save(j: JsonWriter): Unit
    def load(j: JsonReader): Unit
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
    def currentValue(gl: GLInit): GLStoredResult[T]
  }

  class UnnamedPicker[V](override val control: AdapterView[ListAdapter]) extends SavedControl with GLControl[V] with SelectedListener {
    type LP = LazyPicker[V]
    type U = AdapterView[LP]
    override def currentValue(gl: GLInit): GLStoredResult[V] = {
      Log.i("picker", s"getting value at idx ${selected}: '${adapter.lazified(selected).name}'")
      adapter.getState(selected, gl)
    }
    var selectedName = ""
    private var adapter: LP = null
    def setAdapter(a: LP) = {
      adapter = a
      control.setAdapter(a)
    }
    override def restoreState(): Unit = {
      Log.i("picker", s"restoring unnamedpicker state to '${selectedName}'")
      selected = this.adapter.lazified.indexWhere(_.name == selectedName) match {
        case -1 => 0
        case  x => x
      }
      if (enabled) this.control.performItemClick(null, selected, selected)
    }
    override def updateState() = selectedName = selected match {
      case AdapterView.INVALID_POSITION => ""
      case x => adapter.lazified(x).name
    }
    override def save(j: JsonWriter) = {
      j.beginObject()
        j.name("enabled").value(enabled)
        j.name("selectedName").value(selectedName)
      j.endObject()
    }
    override def load(j: JsonReader) = {
      j.beginObject()
        while (j.hasNext()) j.nextName() match {
          case "enabled" => enabled = j.nextBoolean()
          case "selectedName" => selectedName = j.nextString()
        }
      j.endObject()
    }
  }

  class FixedIndexPicker(override val control: AdapterView[ListAdapter]) extends SavedControl with SelectedListener {
    override def restoreState(): Unit = {
      Log.i("picker", s"clicking ${selected} in sidebar")
      this.control.performItemClick(null, selected, selected)
    }
    override def updateState() = { }
    override def save(j: JsonWriter) = j.value(selected)
    override def load(j: JsonReader) = { selected = j.nextInt() }
  }

  class UnnamedUnpicker[T](var value: Option[T] = None) extends SavedControl with GLControl[T] {
    override def save(j: JsonWriter) = j.value(enabled)
    override def load(j: JsonReader) = { enabled = j.nextBoolean() }
    override def currentValue(gl: GLInit): GLStoredResult[T] = {
      Log.i("picker", "getting unpicker value")
      value.getOrElse(throw new GLException("No value present?"))
      value match {
        case None => Left("No value present?")
        case Some(x) => Right(x)
      }
    }
  }

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
