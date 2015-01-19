package com.github.wartman4404.gldraw

import android.os.Bundle
import android.content.Context
import android.view.View
import android.widget.{AdapterView, Adapter, GridView, ListAdapter, Spinner, SpinnerAdapter}
import android.util.Log
import android.graphics.Bitmap

import java.io.{InputStream, OutputStream, OutputStreamWriter, BufferedWriter}

import unibrush.{UniBrush, UniBrushSource}

import PaintControls._
import GLResultTypeDef._
import java.io.{StringReader, StringWriter}
import android.util.{JsonReader, JsonWriter}

import com.larswerkman.holocolorpicker.{ColorPicker, ScaleBar}

class PaintControls
  (val animpicker: UP[CopyShader], val brushpicker: UP[Texture], val paintpicker: UP[PointShader], val interppicker: UP[LuaScript], val unipicker: UP[UniBrush], val copypicker: UUP[CopyShader], val colorpicker: ColorUnpicker, val scalebar: ScaleUnpicker, val rotation: RotationUnpicker) {

  val namedPickers: Map[String, SavedControl] = Map(
    "anim" -> animpicker,
    "brush" -> brushpicker,
    "paint" -> paintpicker,
    "interp" -> interppicker,
    "unibrush" -> unipicker,
    "copy" -> copypicker,
    "color" -> colorpicker,
    "scale" -> scalebar,
    "rotation" -> rotation
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
      val name = reader.nextName()
      if (namedPickers.contains(name)) {
        namedPickers(name).load(reader)
      } else {
        reader.skipValue()
      }
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
  type LAV = AdapterView[SpinnerAdapter]
  type UP[U] = UnnamedPicker[U]
  type UUP[T] = UnnamedUnpicker[T]
  type FIP = FixedIndexPicker
  def apply
  (animpicker: LAV, brushpicker: LAV, paintpicker: LAV, interppicker: LAV, unipicker: LAV, colorpicker: ColorPicker, scalebar: ScaleBar) = {
    new PaintControls (
      new UnnamedPicker[CopyShader](animpicker),
      new UnnamedPicker[Texture](brushpicker),
      new UnnamedPicker[PointShader](paintpicker),
      new UnnamedPicker[LuaScript](interppicker),
      new UnnamedPicker[UniBrush](unipicker),
      new UnnamedUnpicker[CopyShader](None),
      new ColorUnpicker(colorpicker),
      new ScaleUnpicker(scalebar),
      new RotationUnpicker(-1))
  }

  trait SavedControl {
    def save(j: JsonWriter): Unit
    def load(j: JsonReader): Unit
    def restoreState() { }
    def updateState() { }
  }

  trait SelectedListener {
    type AdapterType <: Adapter
    val control: AdapterView[AdapterType]
    var selected = AdapterView.INVALID_POSITION
    def setListener(cb: (View, Int) => Unit) = {
      control.setOnItemSelectedListener(new AdapterView.OnItemSelectedListener() {
        override def onItemSelected(parent: AdapterView[_], v: View, pos: Int, id: Long) = {
          selected = pos
          cb(v, pos)
        }
        override def onNothingSelected(parent: AdapterView[_]) = {
          selected = AdapterView.INVALID_POSITION
        }
      })
    }
  }

  trait GLControl[T] {
    var enabled: Boolean = true
    def currentValue(gl: GLInit): GLStoredResult[T]
  }

  class UnnamedPicker[V](override val control: AdapterView[SpinnerAdapter]) extends SavedControl with GLControl[V] with SelectedListener {
    override type AdapterType = SpinnerAdapter
    type LP = LazyPicker[V]
    type U = AdapterView[LP]
    override def currentValue(gl: GLInit): GLStoredResult[V] = {
      adapter.getState(selected, gl)
    }
    var selectedName = ""
    private var adapter: LP = null
    def setAdapter(a: LP) = {
      adapter = a
      control.setAdapter(a)
    }
    override def restoreState(): Unit = {
      selected = this.adapter.lazified.indexWhere(_.name == selectedName) match {
        case -1 => 0
        case  x => x
      }
      if (enabled) this.control.setSelection(selected)
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
    override type AdapterType = ListAdapter
    override def restoreState(): Unit = {
      this.control.setSelection(selected)
    }
    override def updateState() = { }
    override def save(j: JsonWriter) = j.value(selected)
    override def load(j: JsonReader) = { selected = j.nextInt() }
  }

  class UnnamedUnpicker[T](var value: Option[T] = None) extends SavedControl with GLControl[T] {
    override def save(j: JsonWriter) = j.value(enabled)
    override def load(j: JsonReader) = { enabled = j.nextBoolean() }
    override def currentValue(gl: GLInit): GLStoredResult[T] = {
      value.getOrElse(throw new GLException("No value present?"))
      value match {
        case None => Left("No value present?")
        case Some(x) => Right(x)
      }
    }
  }

  class RotationUnpicker(var value: Int = -1) extends SavedControl {
    override def save(j: JsonWriter) = j.value(value)
    override def load(j: JsonReader) = { value = j.nextInt() }
    override def restoreState() = { }
  }

  class ColorUnpicker(val color: ColorPicker) extends SavedControl {
    override def save(j: JsonWriter) = j.value(color.getColor())
    override def load(j: JsonReader) = { color.setColor(j.nextInt()) }
  }

  class ScaleUnpicker(val scale: ScaleBar) extends SavedControl {
    override def save(j: JsonWriter) = j.value(scale.getScale())
    override def load(j: JsonReader) = { scale.setScale(j.nextDouble().asInstanceOf[Float]) }
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
