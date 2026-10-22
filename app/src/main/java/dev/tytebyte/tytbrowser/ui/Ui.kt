package dev.tytebyte.tytbrowser.ui

import android.content.Context
import android.content.res.ColorStateList
import android.graphics.Color
import android.graphics.Typeface
import android.graphics.drawable.Drawable
import android.graphics.drawable.GradientDrawable
import android.graphics.drawable.RippleDrawable
import android.util.TypedValue
import android.view.Gravity
import android.view.View
import android.view.ViewGroup
import android.widget.FrameLayout
import android.widget.ImageView
import android.widget.LinearLayout
import android.widget.TextView

/** Палитра по темам: 0 — светлая, 1 — тёмная, 2 — AMOLED. Приватные вкладки всегда тёмно-фиолетовые. */
class Pal(val mode: Int, val priv: Boolean = false) {
    val dark = mode != 0 || priv
    val bg: Int = when {
        priv -> 0xFF1A1226.toInt()
        mode == 2 -> 0xFF000000.toInt()
        mode == 1 -> 0xFF171717.toInt()
        else -> 0xFFFFFFFF.toInt()
    }
    val bar: Int = when {
        priv -> 0xFF211733.toInt()
        mode == 2 -> 0xFF000000.toInt()
        mode == 1 -> 0xFF1F1F1F.toInt()
        else -> 0xFFFFFFFF.toInt()
    }
    val field: Int = when {
        priv -> 0xFF2D2145.toInt()
        mode == 2 -> 0xFF111111.toInt()
        mode == 1 -> 0xFF2B2B2B.toInt()
        else -> 0xFFF0F1F3.toInt()
    }
    val card: Int = when {
        priv -> 0xFF251A38.toInt()
        mode == 2 -> 0xFF0C0C0C.toInt()
        mode == 1 -> 0xFF242424.toInt()
        else -> 0xFFF6F7F8.toInt()
    }
    val text: Int = if (dark) 0xFFF2F2F2.toInt() else 0xFF1B1B1B.toInt()
    val muted: Int = if (dark) 0xFF9A9A9A.toInt() else 0xFF6B6B6B.toInt()
    val border: Int = if (dark) 0xFF2E2E2E.toInt() else 0xFFE1E3E6.toInt()
    val accent: Int = if (priv) 0xFFB388FF.toInt() else 0xFF3B82F6.toInt()
    val onAccent: Int = 0xFFFFFFFF.toInt()
    val danger: Int = 0xFFEF5350.toInt()
    val ok: Int = 0xFF22C55E.toInt()
    val ripple: Int = if (dark) 0x33FFFFFF else 0x22000000
}

fun Context.dp(v: Int): Int = (v * resources.displayMetrics.density + 0.5f).toInt()

fun shape(fill: Int, radiusPx: Float, stroke: Int = 0, strokePx: Int = 0): GradientDrawable =
    GradientDrawable().apply {
        setColor(fill)
        cornerRadius = radiusPx
        if (strokePx > 0) setStroke(strokePx, stroke)
    }

fun ripple(color: Int, content: Drawable?, mask: Drawable? = null): Drawable =
    RippleDrawable(ColorStateList.valueOf(color), content, mask ?: content?.let { shape(Color.WHITE, 0f) } ?: shape(Color.WHITE, 0f))

fun Context.text(
    s: CharSequence,
    sizeSp: Float = 14f,
    color: Int = Color.BLACK,
    bold: Boolean = false,
    gravity: Int = Gravity.START or Gravity.CENTER_VERTICAL,
): TextView = TextView(this).apply {
    text = s
    setTextSize(TypedValue.COMPLEX_UNIT_SP, sizeSp)
    setTextColor(color)
    if (bold) typeface = Typeface.DEFAULT_BOLD
    this.gravity = gravity
    includeFontPadding = false
}

/** Круглая кнопка-иконка из символа. */
fun Context.iconButton(glyph: String, pal: Pal, sizeDp: Int = 40, textSp: Float = 20f, onClick: () -> Unit): TextView =
    text(glyph, textSp, pal.text, false, Gravity.CENTER).apply {
        layoutParams = LinearLayout.LayoutParams(dp(sizeDp), dp(sizeDp))
        background = ripple(pal.ripple, null, shape(Color.WHITE, dp(sizeDp).toFloat()))
        isClickable = true
        isFocusable = true
        contentDescription = glyph
        setOnClickListener { onClick() }
    }

/** Круглая кнопка с векторной иконкой (тонируется цветом текста палитры). */
fun Context.vectorButton(resId: Int, pal: Pal, description: String, sizeDp: Int = 38, tint: Int = pal.text, onClick: () -> Unit): ImageView =
    ImageView(this).apply {
        setImageResource(resId)
        setColorFilter(tint, android.graphics.PorterDuff.Mode.SRC_IN)
        scaleType = ImageView.ScaleType.CENTER_INSIDE
        setPadding(dp(9), dp(9), dp(9), dp(9))
        layoutParams = LinearLayout.LayoutParams(dp(sizeDp), dp(sizeDp))
        background = ripple(pal.ripple, null, shape(Color.WHITE, dp(sizeDp).toFloat()))
        contentDescription = description
        isClickable = true
        isFocusable = true
        setOnClickListener { onClick() }
    }

fun Context.pillButton(label: String, pal: Pal, filled: Boolean = true, onClick: () -> Unit): TextView =
    text(label, 14f, if (filled) pal.onAccent else pal.accent, true, Gravity.CENTER).apply {
        setPadding(dp(18), dp(10), dp(18), dp(10))
        background = ripple(
            pal.ripple,
            shape(if (filled) pal.accent else Color.TRANSPARENT, dp(12).toFloat(), pal.accent, if (filled) 0 else dp(1)),
            shape(Color.WHITE, dp(12).toFloat()),
        )
        isClickable = true
        setOnClickListener { onClick() }
    }

fun Context.divider(pal: Pal): View = View(this).apply {
    setBackgroundColor(pal.border)
    layoutParams = LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, 1)
}

fun matchWrap() = LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT)
fun matchMatch() = FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT)
fun weighted(w: Float = 1f) = LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, w)
