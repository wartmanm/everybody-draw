package com.github.wartman4404.gldraw

import android.app.AlertDialog
import android.content.Context
import android.view._
import android.widget._
import TypedResource.view2typed
import android.util.Log

case class CreditItem(title: String, license: String)

object Credits extends AndroidImplicits {
  val creditArray = CreditData.creditArray

  class CreditAdapter(context: Context) extends BaseAdapter {
    val inflater = LayoutInflater.from(context)
    override def getCount(): Int = creditArray.length
    override def getItem(pos: Int): CreditItem = creditArray(pos)
    override def getItemId(pos: Int): Long = pos
    override def getView(position: Int, convertView: View, parent: ViewGroup) = {
      val view = Option(convertView).getOrElse(inflater.inflate(R.layout.credit_list_item, parent, false))
      val credits = getItem(position)
      view.findView(TR.credit_list_item_title).setText(credits.title)
      view.findView(TR.credit_list_item).setText(credits.license)
      view
    }
  }

  def displayCredits(context: Context) = {
    val content = LayoutInflater.from(context)
      .inflate(R.layout.credit_list, null)
      .asInstanceOf[ListView]
    content.findView(TR.credit_list).setAdapter(new CreditAdapter(context))
    new AlertDialog.Builder(context)
    .setView(content)
    .setTitle("Everybody Draws!")
    .setPositiveButton("OK", () => {})
    .show()
  }
}
