package com.github.wartman4404.gldraw

import _root_.android.app.Activity
import _root_.android.os.Bundle

import android.widget._
import android.view._
import android.graphics.{SurfaceTexture, Bitmap}
import android.content.{Context, Intent}
import android.content.res.Configuration
import android.app.AlertDialog

import java.io.{BufferedInputStream}
import java.io.{OutputStream, FileOutputStream, BufferedOutputStream}
import java.io.{File, IOException}
import java.util.Date

import android.util.Log

import scala.collection.mutable

import com.ipaulpro.afilechooser.utils.FileUtils

import unibrush.{UniBrush, Layer}

import resource._

import com.larswerkman.holocolorpicker.{ColorPicker, ScaleBar}

import scala.concurrent.ExecutionContext
import scala.concurrent.Future
import java.util.concurrent.Executors

import PaintControls.UnnamedPicker
import PaintControls.GLControl



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
    unipicker = findView(TR.unipicker),
    sidebar = controldrawer
  )

  lazy val drawerParent = findView(TR.drawer_parent)
  lazy val controlflipper = findView(TR.controlflipper)
  lazy val controldrawer = findView(TR.control_drawer)
  lazy val sidebar = findView(TR.sidebar_parent)
  lazy val drawerToggle = new android.support.v7.app.ActionBarDrawerToggle(
      this, drawerParent, R.string.sidebar_open, R.string.sidebar_close)
  lazy val sidebarAdapter = new SidebarAdapter()
  lazy val undoButton = findView(TR.undo_button)
  lazy val redoButton = findView(TR.redo_button)
  lazy val clearButton = findView(TR.clear_button)
  lazy val loadButton = findView(TR.load_button)
  lazy val saveButton = findView(TR.save_button)
  lazy val colorPicker = findView(TR.brush_colorpicker_main)

  var textureThread: Option[TextureSurfaceThread] = None

  private var savedBitmap: Option[Bitmap] = None

  lazy val saveThread = ExecutionContext.fromExecutor(Executors.newSingleThreadExecutor())

  @native protected def nativeAppendMotionEvent(handler: MotionEventProducer, m: MotionEvent): Unit
  @native protected def nativePauseMotionEvent(handler: MotionEventProducer): Unit

  // TODO: actually clean up
  var handlers: Option[MotionEventHandlerPair] = None

  def createTextureThread(handlers: MotionEventHandlerPair)(s: SurfaceTexture, x: Int, y: Int): Unit = {
    Log.i("main", "got surfacetexture");
    val thread = new TextureSurfaceThread(s, handlers.consumer, onTextureThreadStarted(x,y, handlers.producer), onTextureThreadError);
    thread.start()
    Log.i("main", "started thread");
  }

  var undoCount: Int = 0
  var undoPos: Int = 0

  class MainUndoListener() extends UndoCallback() {
    override def undoBufferChanged(newSize: Int): Unit = {
      Log.i("main", s"new undo buffer size: ${newSize}")
      undoCount = newSize
      undoPos = newSize - 1
      runOnUiThread(() => {
        updateUndoButtons()
      })
    }
  }

  def moveUndo(offset: Int) = {
    val newPos = undoPos + offset
    if (newPos >= 0 && newPos < undoCount) {
      for (thread <- textureThread) {
        undoPos = newPos
        updateUndoButtons()
        thread.withGL(gl => {
          thread.loadUndo(gl, newPos)
        })
      }
    }
  }

  def updateUndoButtons() = {
    redoButton.setEnabled(undoCount - undoPos > 1)
    undoButton.setEnabled(undoPos > 0)
  }

  val onTextureThreadStarted = (x: Int, y: Int, producer: MotionEventProducer) => (thread: TextureSurfaceThread) => this.runOnUiThread(() => {
    Log.i("main", "got handler")
    textureThread = Some(thread)
    val undoCallback = new MainUndoListener()
    thread.beginGL(x, y, onTextureCreated(thread, producer) _, undoCallback)
    //thread.startFrames() // FIXME is this needed?
    Log.i("main", "sent begin_gl message")
    ()
  })

  // runs on gl thread
  def onTextureCreated(thread: TextureSurfaceThread, producer: MotionEventProducer)(gl: GLInit) = {
    thread.initScreen(gl, savedBitmap)
    savedBitmap = None
    thread.startFrames(gl)
    populatePickers(producer, thread, gl)
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

    controls.sidebar.control.setAdapter(sidebarAdapter)
    controls.sidebar.setListener((v: View, pos: Int) => {

      if (pos >= 0 && pos < sidebarAdapter.sidebarControls.length) {
        sidebarAdapter.sidebarControls(pos).onClick(pos)
      }
    })
    
    updateUndoButtons()
    undoButton.setOnClickListener(() => moveUndo(-1))
    redoButton.setOnClickListener(() => moveUndo(1))

    saveButton.setOnClickListener(saveFile _)
    loadButton.setOnClickListener(loadFile _)
    clearButton.setOnClickListener(() => textureThread.foreach(_.clearScreen()))

    colorPicker.addSVBar(findView(TR.brush_colorpicker_svbar))
    val scaleBar = findView(TR.brush_colorpicker_scalebar)
    colorPicker.addScaleBar(scaleBar)
    colorPicker.setShowOldCenterColor(false)
    colorPicker.setOnColorChangedListener(new ColorPicker.OnColorChangedListener() {
      override def onColorChanged(color: Int) = {
        textureThread.foreach(t => t.withGL(gl => t.setBrushColor(gl, color)))
      }
    })
    scaleBar.setOnScaleChangedListener(new ScaleBar.OnScaleChangedListener() {
      override def onScaleChanged(size: Float) = {
        textureThread.foreach(t => t.withGL(gl => t.setBrushSize(gl, size)))
      }
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
      case R.id.menu_credits => Toast.makeText(this, "Soon.", Toast.LENGTH_LONG).show()
      case R.id.menu_debug => showDebugMessagebox()
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
        savedBitmap = Some(DrawFiles.decodeBitmap(Bitmap.Config.ARGB_8888)(input))
        val input2 = MainActivity.this.openFileInput("status")
        controls.load(input2)
      }
    } catch {
      case e @ (_: IOException | _: GLException) => { 
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

  def populatePicker[U, T <: (String, (GLInit)=>GLResult[U])](picker: UnnamedPicker[U], arr: Array[T], cb: (GLInit, U)=>Unit, thread: TextureSurfaceThread) = {
    val adapter = new LazyPicker(this, thread, arr)
    picker.setAdapter(adapter)
    picker.setListener((view: View, pos: Int) => {
      //Log.i("main", s"Item selected for {picker.name}! {pos}")
      thread.withGL(gl => adapter.getState(pos, gl) match {
        case Right(value) => cb(gl, value)
        case Left(errmsg) => {
          MainActivity.this.runOnUiThread(() => {
            Toast.makeText(MainActivity.this, "unable to load item!\n" + errmsg, Toast.LENGTH_LONG).show()
            picker.control.performItemClick(null, 0, 0)
            adapter.notifyDataSetChanged()
            ()
          })
        }
      })
    })
  }

  def unloadInterpolatorSynchronized(thread: TextureSurfaceThread, producer: MotionEventProducer, gl: GLInit) = {
    val notify = new Object()
    notify.synchronized {
      runOnUiThread(() => {
        nativePauseMotionEvent(producer)
        Log.i("main", "loading interpolator - paused motion events")
        notify.synchronized {
          notify.notify()
        }
      })
      Log.i("main", "loading interpolator - waiting for pause")
      notify.wait()
    }
    Log.i("main", "loading interpolator - finishing lua script")
    try {
      thread.finishLuaScript(gl)
    }
    catch { case _: LuaException => { } }
  }

  def loadInterpolatorSynchronized(thread: TextureSurfaceThread, producer: MotionEventProducer) =
  (gl: GLInit, script: LuaScript) => {
    unloadInterpolatorSynchronized(thread, producer, gl)
    thread.setInterpScript(gl, script)
  }

  def populatePickers(producer: MotionEventProducer, thread: TextureSurfaceThread, gl: GLInit) = {
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
      val interpLoader = loadInterpolatorSynchronized(thread, producer)
      populatePicker(controls.brushpicker, brushes, loadBrush(thread), thread)
      populatePicker(controls.animpicker, anims,  thread.setAnimShader _, thread)
      populatePicker(controls.paintpicker, paints,  thread.setPointShader _, thread)
      populatePicker(controls.interppicker, interpscripts,  interpLoader, thread)
      populatePicker(controls.unipicker, unibrushes, loadUniBrush(thread, producer), thread)
      controls.copypicker.value = thread.outputShader
      controls.restoreState()
    })
  }

  // TODO: fewer callbacks
  def loadUniBrushControls(unibrush: UniBrush) = {
    runOnUiThread(() => {
      sidebarAdapter.updateUnibrush(unibrush)
    })
  }

  def loadBrush(thread: TextureSurfaceThread) = (gl: GLInit, bmtx: Texture) => {
    thread.setBrushTexture(gl, bmtx)
    runOnUiThread(() => {
      colorPicker.setNewCenterBitmap(bmtx.bitmap)
    })
  }

  def loadUniBrush(thread: TextureSurfaceThread, producer: MotionEventProducer) =
  (gl: GLInit, unibrush: UniBrush) => {
    Log.i("main", "loading unibrush")
    def getSelectedValue[T](picker: GLControl[T]): Option[T] = {
      // return None if the control is already active, or we're trying to restore a missing value
      // TODO: the missing-value part is probably busted
      if (picker.enabled) {
        None
      } else {
        val tmp: GLStoredResult[T] = picker.currentValue(gl)
        tmp match {
          case Left(msg) => {
            runOnUiThread(() => {
              Toast.makeText(MainActivity.this, "unable to load old control!" + msg, Toast.LENGTH_LONG).show()
            })
            Log.i("main", s"unable to load old control: ${msg}")
            None
          }
          case Right(value) => {
            val tmp: T = value
            Some(tmp)
          }
        }
      }
    }
    //Log.i("main", s"copypicker is enabled: ${controls.copypicker.enabled}")
    Log.i("unibrush", "loading unibrushes and old values...")
    Log.i("unibrush", "loading brush")
    val brush: Option[Texture] = unibrush.brush.orElse(getSelectedValue(controls.brushpicker))
    Log.i("unibrush", "loading anim")
    val anim = unibrush.baseanimshader.orElse(getSelectedValue(controls.animpicker))
    Log.i("unibrush", "loading point")
    val point = unibrush.basepointshader.orElse(getSelectedValue(controls.paintpicker))
    Log.i("unibrush", "loading copy")
    val copy = unibrush.basecopyshader.orElse(getSelectedValue(controls.copypicker))
    Log.i("unibrush", "loading interp")
    val interp = unibrush.interpolator.orElse(getSelectedValue(controls.interppicker))
    Log.i("unibrush", "loading unibrush!")
    
    // Unconditionally call ondone() in the interpolator to write layers, etc
    // This runs the old interpolator and so must run under the old state.
    unloadInterpolatorSynchronized(thread, producer, gl)
    Log.i("unibrush", s"should have unloaded interpolator, which is ${interp} (unibrush interp is ${unibrush.interpolator})")
    thread.clearLayers(gl)
    for (layer <- unibrush.layers) {
      thread.addLayer(gl, layer.copyshader, layer.pointshader, layer.pointsrc)
    }
    Log.i("unibrush", "set up layers!")
    brush.foreach(thread.setBrushTexture(gl, _))
    anim.foreach(thread.setAnimShader(gl, _))
    point.foreach(thread.setPointShader(gl, _))
    copy.foreach(thread.setCopyShader(gl, _))
    interp.foreach(thread.setInterpScript(gl, _))
    Log.i("unibrush", "done loading unibrush!")
    loadUniBrushControls(unibrush) // now that we're done, update which controls are enabled
    ()
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
        val bitmap = (try {
          val tmp: Option[Bitmap] = DrawFiles.withFileStream(new File(path))
          .acquireAndGet(fs => Some(DrawFiles.decodeBitmap(Bitmap.Config.ARGB_8888)(fs)))
          tmp
        } catch {
          case e: Exception => None
        })
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
    drawerParent.closeDrawer(sidebar)
  }
  def hideControls() = {
    controlflipper.setVisibility(View.INVISIBLE)
    drawerParent.closeDrawer(sidebar)
  }

  def showDebugMessagebox() {
    for (thread <- textureThread) thread.withGL(gl => {
       def getSource[T,U](gl: GLInit, control: GLControl[T], cb: (GLInit, T)=>U, default: U) = {
         control.currentValue(gl).right.toOption.map(cb(gl, _)).getOrElse(default)
       }
       val animdebug = getSource(gl, controls.animpicker, CopyShader.getSource, ("", ""))
       val copydebug = getSource(gl, controls.copypicker, CopyShader.getSource, ("", ""))
       val paintdebug = getSource(gl, controls.paintpicker, PointShader.getSource, ("", ""))
       val interpdebug = getSource(gl, controls.interppicker, LuaScript.getSource, "")

       val strs = Array(
         animdebug._1 + "\n\n" + animdebug._2,
         copydebug._1 + "\n\n" + copydebug._2,
         paintdebug._1 + "\n\n" + paintdebug._2,
         interpdebug)
       MainActivity.this.runOnUiThread(() => {
         val text = new TextView(this)
         text.setText(strs.mkString("-------"))
         new AlertDialog.Builder(this)
         .setView(text)
         .setTitle("debug")
         .setPositiveButton("Done", () => {})
         .show()
         ()
       })
    })
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
    val copyShaderControl = new SidebarEntryPicker("Overlay", controls.copypicker, (u: UniBrush) => u.basecopyshader)
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
      copyShaderControl.updateForUnibrush(unibrush)

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
    class SidebarEntryPicker[T](val name: String, picker: GLControl[_], getUnibrushValue: (UniBrush) => Option[T]) extends SidebarEntry {
      override def enabled = picker.enabled
      override def updateForUnibrush(u: UniBrush) = {
        val oldstate = enabled
        picker.enabled = getUnibrushValue(u).isEmpty
        Log.i("main", s"${if (enabled) "enabling" else "disabling"} control ${name} for unibrush (was: ${if (oldstate) "enabled" else "disabled"})")
      }
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
