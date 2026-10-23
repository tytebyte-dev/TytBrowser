package dev.tytebyte.tytbrowser

import android.content.Context
import android.os.Handler
import android.os.Looper
import java.io.File
import java.net.HttpURLConnection
import java.net.URL
import java.text.DateFormat
import java.util.Date

/** Базы geoip.dat / geosite.dat для правил `geoip:` и `geosite:`. Лежат в папке приложения и подхватываются ядром Xray. */
object GeoData {
    /** Сборка v2ray-rules с российскими списками (geosite:category-ru, ru-blocked, geoip:ru и др.). */
    const val BASE_URL = "https://github.com/runetfreedom/russia-v2ray-rules-dat/releases/latest/download/"
    val FILES = listOf("geoip.dat", "geosite.dat")
    private const val MIN_SIZE = 50_000L

    fun dir(ctx: Context): File = File(ctx.filesDir, "geo").apply { mkdirs() }

    fun has(ctx: Context, name: String): Boolean = File(dir(ctx), name).let { it.isFile && it.length() >= MIN_SIZE }

    fun hasAll(ctx: Context): Boolean = FILES.all { has(ctx, it) }

    fun status(ctx: Context): String {
        val f = File(dir(ctx), FILES[0])
        if (!hasAll(ctx)) return "не загружены"
        val mb = FILES.sumOf { File(dir(ctx), it).length() } / 1024.0 / 1024.0
        return "%.1f МБ, обновлены %s".format(mb, DateFormat.getDateInstance(DateFormat.SHORT).format(Date(f.lastModified())))
    }

    /** Скачивает обе базы напрямую (не через прокси). [done] вызывается в главном потоке: null — успех, иначе текст ошибки. */
    fun update(ctx: Context, done: (String?) -> Unit) {
        val app = ctx.applicationContext
        val ui = Handler(Looper.getMainLooper())
        Thread {
            val err = try {
                val dir = dir(app)
                for (name in FILES) {
                    val tmp = File(dir, "$name.part")
                    val c = URL(BASE_URL + name).openConnection() as HttpURLConnection
                    c.connectTimeout = 15_000
                    c.readTimeout = 30_000
                    c.instanceFollowRedirects = true
                    try {
                        if (c.responseCode != 200) error("$name: HTTP ${c.responseCode}")
                        c.inputStream.use { i -> tmp.outputStream().use { o -> i.copyTo(o) } }
                    } finally { c.disconnect() }
                    if (tmp.length() < MIN_SIZE) { tmp.delete(); error("$name: файл слишком мал") }
                    val dst = File(dir, name)
                    if (!tmp.renameTo(dst)) { tmp.copyTo(dst, overwrite = true); tmp.delete() }
                }
                null
            } catch (e: Throwable) { e.message ?: e.javaClass.simpleName }
            ui.post { done(err) }
        }.start()
    }
}
