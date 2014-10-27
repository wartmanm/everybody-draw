package com.github.wartman4404.gldraw

import _root_.android.app.Activity
import _root_.android.os.Bundle

import android.widget._
import android.view._
import android.graphics.{SurfaceTexture, Bitmap}
import android.content.{Context, Intent}
import android.content.res.Configuration
import android.opengl.GLException

import java.io.{BufferedInputStream}
import java.io.{OutputStream, FileOutputStream, BufferedOutputStream}
import java.io.{File, IOException}
import java.util.Date

import android.util.Log

import scala.collection.mutable

import com.ipaulpro.afilechooser.utils.FileUtils

import unibrush.{UniBrush, Layer}

import resource._

import scala.concurrent.ExecutionContext
import scala.concurrent.Future
import java.util.concurrent.Executors

import PaintControls.UnnamedPicker
import PaintControls.SavedControl



class MainActivity extends Activity with TypedActivity with AndroidImplicits {
  import MainActivity._
  import MainActivity.Constants._
  import GLResultTypeDef._

  lazy val content = new TextureView(this)
  lazy val contentframe = findView(TR.textureviewframe)

  lazy val controls = PaintControls(
    brushpicker = findView(TR.brushpicker),
    animpicker = findView(TR.animpicker),
    paintpicker = findView(TR.paintpicker),
    interppicker = findView(TR.interppicker),
    unipicker = findView(TR.unipicker))

  lazy val drawerParent = findView(TR.drawer_parent)
  lazy val controlflipper = findView(TR.controlflipper)
  lazy val controldrawer = findView(TR.control_drawer)
  lazy val drawerToggle = new android.support.v7.app.ActionBarDrawerToggle(
      this, drawerParent, R.string.sidebar_open, R.string.sidebar_close)
  lazy val sidebarAdapter = new SidebarAdapter()

  var textureThread: Option[TextureSurfaceThread] = None

  private var savedBitmap: Option[Bitmap] = None

  lazy val saveThread = ExecutionContext.fromExecutor(Executors.newSingleThreadExecutor())

  @native protected def nativeAppendMotionEvent(handler: MotionEventProducer, m: MotionEvent): Unit

  // TODO: actually clean up
  var handlers: Option[MotionEventHandlerPair] = None

  def createTextureThread(handlers: MotionEventHandlerPair)(s: SurfaceTexture, x: Int, y: Int): Unit = {
    Log.i("main", "got surfacetexture");
    val thread = new TextureSurfaceThread(s, handlers.consumer, onTextureThreadStarted(x,y, handlers.producer), onTextureThreadError);
    thread.start()
    Log.i("main", "started thread");
  }

  val onTextureThreadStarted = (x: Int, y: Int, producer: MotionEventProducer) => (thread: TextureSurfaceThread) => this.runOnUiThread(() => {
    Log.i("main", "got handler")
    textureThread = Some(thread)
    thread.beginGL(x, y, onTextureCreated(thread, producer) _)
    thread.startFrames()
    Log.i("main", "sent begin_gl message")
    ()
  })

  // runs on gl thread
  def onTextureCreated(thread: TextureSurfaceThread, producer: MotionEventProducer)() = {
    thread.initScreen(savedBitmap)
    savedBitmap = None
    thread.startFrames()
    populatePickers()
    content.setOnTouchListener(createViewTouchListener(producer))
    Log.i("main", "set ontouch listener")
  }

  def createViewTouchListener(producer: MotionEventProducer) = new View.OnTouchListener() {
    override def onTouch(v: View, evt: MotionEvent) = {
      nativeAppendMotionEvent(producer, evt)
      true
    }
  }

  override def onCreate(bundle: Bundle) {
    Log.i("main", "oncreate")
    System.loadLibrary("gl-stuff")
    handlers = Some(MotionEventHandlerPair.init())

    super.onCreate(bundle)
    setContentView(R.layout.activity_main)

    controldrawer.setAdapter(sidebarAdapter)
    controldrawer.setOnItemClickListener((v: View, pos: Int) => {
        sidebarAdapter.sidebarControls(pos).onClick(pos)
      })

    drawerParent.setDrawerListener(drawerToggle)
    getActionBar().setDisplayHomeAsUpEnabled(true)
    getActionBar().setHomeButtonEnabled(true)

    // TODO: deal with rotation better
    Option(bundle) match {
      case Some(inState) => {
        Log.i("main", "got bundle to restore")
        savedBitmap = Option(inState.getParcelable("screen"))
        controls.load(inState)
      }
      case None => {
        loadFromFile()
      }
    }
  }

  override protected def onPostCreate(bundle: Bundle) = {
    super.onPostCreate(bundle)
    drawerToggle.syncState()
  }

  override def onConfigurationChanged(config: Configuration) = {
    super.onConfigurationChanged(config)
    drawerToggle.onConfigurationChanged(config)
  }

  override def onOptionsItemSelected(item: MenuItem): Boolean = {
    if (drawerToggle.onOptionsItemSelected(item)) true
    else item.getItemId() match {
      case R.id.menu_save => saveFile()
      case R.id.menu_load => loadFile()
      case R.id.menu_replay => startReplay()
      case R.id.menu_clear => textureThread.foreach(_.clearScreen())
      case _ => return super.onOptionsItemSelected(item)
    }
    true
  }

  override def onStart() = {
    Log.i("main", "onStart")
    super.onStart()
    handlers.foreach(h => {
        content.setSurfaceTextureListener(new TextureListener(createTextureThread(h) _))
        contentframe.addView(content)
      })

  }
  
  // FIXME the texture thread might not be ready yet
  // although, i guess onTextureCreated handles that case?
  override def onResume() = {
    super.onResume()
    textureThread.foreach(_.startFrames())
    Log.i("main", "resumed!")
  }

  override def onPause() = {
    textureThread.foreach(_.stopFrames())
    super.onPause()
    prepareForSave()
    Log.i("main", "paused!")
  }

  protected override def onSaveInstanceState(outState: Bundle) = {
    Log.i("main", "saving instance state")
    super.onSaveInstanceState(outState)
    savedBitmap.synchronized {
      savedBitmap.foreach(bitmap => {
          Log.i("main", "saved bitmap to bundle")
          outState.putParcelable("screen", bitmap)
        })
    }
    controls.save(outState)
  }

  override protected def onStop() = {
    super.onStop()
    Log.i("main", "onStop");
    content.setOnTouchListener(null)
    // (textureview does its own cleanup, see SurfaceTextureListener.onSurfaceTextureDestroyed())
    // TODO: is this the right order? probably not
    contentframe.removeAllViews()
    saveLocalState()
    finishEGLCleanup()
    // TODO: is this necessary?
    textureThread.foreach(_.join())
    textureThread = None
  }

  override protected def onDestroy() = {
    super.onDestroy()
    handlers.foreach(MotionEventHandlerPair.destroy _)
    handlers = None
  }

  private def prepareForSave() = {
    Log.i("main", "preparing for save")
    for (thread <- textureThread) {
      textureThread.foreach(thread => savedBitmap = Some(thread.getBitmapSynchronized()))
    }
    controls.updateState()
  }

  private def saveBitmapToFile(bitmap: Bitmap, out: OutputStream): Unit = {
    try {
      if (!bitmap.isRecycled()) {
        bitmap.compress(Bitmap.CompressFormat.PNG, 90, out)
      } else {
        Log.i("main", "tried to save recycled bitmap!")
      }
    } catch {
      case e: IOException => {
        Log.i("main", "saving to file failed: %s".format(e))
      }
    }
  }

  private def savePickersToFile() = {
    try {
      val out2 = MainActivity.this.openFileOutput("status", Context.MODE_PRIVATE)
      controls.save(out2)
    } catch {
      case e: IOException => {
        Log.i("main", "saving to file failed: %s".format(e))
      }
    }
  }

  private def loadFromFile() = {
    Log.i("main", "loading from file")
    try {
      for (input <- managed(new BufferedInputStream(MainActivity.this.openFileInput("screen")))) {
        savedBitmap = DrawFiles.decodeBitmap(Bitmap.Config.ARGB_8888)(input).right.toOption
        val input2 = MainActivity.this.openFileInput("status")
        controls.load(input2)
      }
    } catch {
      case e: IOException => { 
        Log.i("main", "loading from file failed: %s".format(e))
      }
    }
  }

  private def saveLocalState() = {
    savedBitmap.foreach(bitmap => {
        Future {
          for (out <- managed(new BufferedOutputStream(
            MainActivity.this.openFileOutput("screen", Context.MODE_PRIVATE)))) {
            saveBitmapToFile(bitmap, out)
          }
        }(saveThread)
      })
    savePickersToFile()
  }

  def populatePicker[U, T <: (String, (GLInit)=>GLResult[U])](picker: UnnamedPicker[U], arr: Array[T], cb: (U)=>Unit, thread: TextureSurfaceThread) = {
    val adapter = new LazyPicker(this, thread, arr)
    picker.setAdapter(adapter)
    picker.control.setOnItemClickListener(new AdapterView.OnItemClickListener() {
        override def onItemClick(parent: AdapterView[_], view: View, pos: Int, id: Long) = {
          picker.selected = pos
          //Log.i("main", s"Item selected for {picker.name}! {pos}")
          adapter.getState(pos, (result: GLResult[U]) => result match {
              case Right(value) => cb(value)
              case Left(errmsg) => {
                MainActivity.this.runOnUiThread(() => {
                  Toast.makeText(MainActivity.this, "unable to load item!\n" + errmsg, Toast.LENGTH_LONG).show()
                  picker.control.performItemClick(null, 0, 0)
                  adapter.notifyDataSetChanged()
                  ()
                })
              }
            })
        }
      })
    picker.restoreState()
  }

  def populatePickers() = {
    for (thread <- textureThread) {
      // TODO: maybe make the save thread load from disk and then hand off to the gl thread?
      // also, have it opportunistically load at least up to that point
      val brushes = DrawFiles.loadBrushes(this).toArray
      val anims = DrawFiles.loadAnimShaders(this).toArray
      val paints = DrawFiles.loadPointShaders(this).toArray
      val interpscripts = DrawFiles.loadScripts(this).toArray
      val unibrushes = DrawFiles.loadUniBrushes(this).toArray
      Log.i("main", s"got ${brushes.length} brushes, ${anims.length} anims, ${paints.length} paints, ${interpscripts.length} interpolation scripts")

      MainActivity.this.runOnUiThread(() => {
        // TODO: make hardcoded shaders accessible a better way
        populatePicker(controls.brushpicker, brushes,  thread.setBrushTexture _, thread)
        populatePicker(controls.animpicker, anims,  thread.setAnimShader _, thread)
        populatePicker(controls.paintpicker, paints,  thread.setPointShader _, thread)
        populatePicker(controls.interppicker, interpscripts,  thread.setInterpScript _, thread)
        populatePicker(controls.unipicker, unibrushes, loadUniBrush _, thread)
        controls.copypicker.currentValue = thread.outputShader
      })
    }
  }

  // TODO: fewer callbacks
  def loadUniBrushControls(unibrush: UniBrush) = {
    runOnUiThread(() => {
      sidebarAdapter.updateUnibrush(unibrush)
    })
  }

  def loadUniBrush(unibrush: UniBrush) = {
    Log.i("main", "loading unibrush")
    def getSelectedValue[T](picker: SavedControl[T]) = {
      // return None if the control is already active, or we're trying to restore a missing value
      // TODO: the missing-value part is probably busted
      if (picker.enabled) None
      else picker.currentValue
    }
    for (thread <- textureThread) {
      loadUniBrushControls(unibrush)
      val brush = unibrush.brush.orElse(getSelectedValue(controls.brushpicker))
      val anim = unibrush.baseanimshader.orElse(getSelectedValue(controls.animpicker))
      val point = unibrush.basepointshader.orElse(getSelectedValue(controls.paintpicker))
      val copy = unibrush.basecopyshader.orElse(getSelectedValue(controls.copypicker))
      val interp = unibrush.interpolator.orElse(getSelectedValue(controls.interppicker))
      thread.loadUniBrush(brush, anim, point, copy, interp, unibrush.layers)
    }
  }

  override def onCreateOptionsMenu(menu: Menu): Boolean = {
    getMenuInflater.inflate(R.menu.main, menu)
    true
  }

  def finishEGLCleanup() {
    textureThread.foreach(thread => {
        thread.cleanupGL()
      })
  }

  def loadFile() {
    val chooser = Intent.createChooser(FileUtils.createGetContentIntent(), "Pick a source image")
    startActivityForResult(chooser, ACTIVITY_CHOOSE_IMAGE) 
  }

  def saveFile() = {
    textureThread.foreach(thread => {
        thread.getBitmap(b => {
            Future {
              val outfile = new File(getExternalFilesDir(null), new Date().toString() + ".png")
              for (outstream <- managed(new BufferedOutputStream(new FileOutputStream(outfile)))) {
                saveBitmapToFile(b, outstream)
              }
            }(saveThread)
          })
      })
  }

  protected override def onActivityResult(requestCode: Int, resultCode: Int, data: Intent) = requestCode match {
    case ACTIVITY_CHOOSE_IMAGE => {
      Log.i("main", s"got activity result: ${data}")
      if (resultCode == Activity.RESULT_OK) {
        val path = FileUtils.getPath(this, data.getData())
        val bitmap = DrawFiles.withFileStream(new File(path)).map(DrawFiles.decodeBitmap(Bitmap.Config.ARGB_8888) _).opt.map(x => x.right.toOption).flatten
        Log.i("main", s"got bitmap ${bitmap}")
        for (b <- bitmap; thread <- textureThread) {
          Log.i("main", "drawing bitmap...")
          thread.drawBitmap(b)
        }
      }
    }
    case _ => {
      Log.i("main", s"got unidentified activity result: ${resultCode}, request code ${requestCode}, data: ${data}")
    }
  }

  def onTextureThreadError(e: Exception) = MainActivity.this.runOnUiThread(() => {
      val prefix = (
        e match {
          case _: LuaException => {
            controls.interppicker.control.performItemClick(null, 0, 0)
            "An error occurred in the interpolator:\n" 
          }
          case _ => "An error occurred:\n" 
        })
      Toast.makeText(MainActivity.this, prefix + e.getMessage(), Toast.LENGTH_LONG).show()
    })

  def startReplay() = {
    for (thread <- textureThread) {
      Log.i("main", "starting replay...")
      thread.beginReplay()
    }
  }
  
  def showControl(pos: Int) = {
    controlflipper.setVisibility(View.VISIBLE)
    controlflipper.setDisplayedChild(pos)
    drawerParent.closeDrawer(controldrawer)
  }
  def hideControls() = {
    controlflipper.setVisibility(View.INVISIBLE)
    drawerParent.closeDrawer(controldrawer)
  }

  class SidebarAdapter() extends BaseAdapter {
    import SidebarAdapter._
    val inflater = LayoutInflater.from(MainActivity.this)
    // must match order of viewflipper children
    val sidebarControls = Array (
      new SidebarEntryPicker("Brush Texture", controls.brushpicker, (u: UniBrush) => u.brush),
      new SidebarEntryPicker("Animation", controls.animpicker, (u: UniBrush) => u.baseanimshader),
      new SidebarEntryPicker("Paint", controls.paintpicker, (u: UniBrush) => u.basepointshader),
      new SidebarEntryPicker("Interpolator", controls.interppicker, (u: UniBrush) => u.interpolator),
      new SidebarEntryPicker("Unibrushes", controls.unipicker, (u: UniBrush) => None),
      new SidebarEntryHider("Hide Controls")
    )
    override def areAllItemsEnabled = false
    override def isEnabled(pos: Int) = sidebarControls(pos).enabled
    override def getCount = sidebarControls.length
    override def getViewTypeCount() = 1
    override def getItem(pos: Int) = sidebarControls(pos)
    override def getItemId(pos: Int) = pos
    override def getView(pos: Int, convertView: View, parent: ViewGroup): View = {
      val view = if (convertView == null) {
        inflater.inflate(android.R.layout.simple_list_item_activated_1, parent, false)
      } else {
        convertView
      }
      val name = view.findViewById(android.R.id.text1).asInstanceOf[TextView]
      val control = sidebarControls(pos)
      val enabled = control.enabled
      name.setText(control.name)
      name.setEnabled(enabled)
      view.setEnabled(enabled)
      view
    }

    def updateUnibrush(unibrush: UniBrush) = {
      for (control <- sidebarControls) {
        control.updateForUnibrush(unibrush)
      }
      this.notifyDataSetChanged()
    }
  }
  object SidebarAdapter {
    trait SidebarEntry {
      def onClick(pos: Int): Unit
      def updateForUnibrush(u: UniBrush): Unit
      def enabled: Boolean
      def name: String
    }
    class SidebarEntryPicker[T](val name: String, picker: UnnamedPicker[_], getUnibrushValue: (UniBrush) => Option[T]) extends SidebarEntry {
      override def enabled = picker.enabled
      override def updateForUnibrush(u: UniBrush) = picker.enabled = getUnibrushValue(u).isEmpty
      override def onClick(pos: Int) = showControl(pos)
    }
    class SidebarEntryHider(val name: String) extends SidebarEntry {
      override def enabled = true
      override def updateForUnibrush(u: UniBrush) = { }
      override def onClick(pos: Int) = hideControls()
    }
  }
}

object MainActivity {

  object Constants {
    final val ACTIVITY_CHOOSE_IMAGE = 0x1;
  }

  class TextureListener(callback: (SurfaceTexture, Int, Int)=>Unit) extends TextureView.SurfaceTextureListener {

    def onSurfaceTextureAvailable(st: android.graphics.SurfaceTexture,  w: Int, h: Int): Unit = {
      callback(st, w, h)
    }
    def onSurfaceTextureDestroyed(st: android.graphics.SurfaceTexture): Boolean = {
      Log.i("main", "got onsurfacetexturedestroyed callback!")
      true
    }
    def onSurfaceTextureSizeChanged(st: android.graphics.SurfaceTexture, w: Int, h: Int): Unit = { }
    def onSurfaceTextureUpdated(st: android.graphics.SurfaceTexture): Unit = { }
  }

  class FrameListener extends SurfaceTexture.OnFrameAvailableListener {
    def onFrameAvailable(st: android.graphics.SurfaceTexture): Unit = { }
  }

  abstract class NamedSidebarControl(val name: String) {
    override def toString() = name
    def onClick(pos: Int)
  }
}
