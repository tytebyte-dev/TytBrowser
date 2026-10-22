package dev.tytebyte.tytbrowser.data

import android.content.ContentValues
import android.content.Context
import android.database.sqlite.SQLiteDatabase
import android.database.sqlite.SQLiteOpenHelper
import java.util.Calendar

data class Bookmark(val id: Long, val title: String, val url: String, val color: Int)
data class HistoryItem(val id: Long, val title: String, val url: String, val visitedAt: Long, val visits: Int)
data class DownloadRec(val dmId: Long, val name: String, val url: String, val at: Long)

/** SQLite: закладки (они же ярлыки быстрого доступа), история посещений, список загрузок. */
class Db(ctx: Context) : SQLiteOpenHelper(ctx.applicationContext, "tytbrowser.db", null, 1) {

    override fun onCreate(db: SQLiteDatabase) {
        db.execSQL("CREATE TABLE bookmarks(id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT NOT NULL, url TEXT NOT NULL UNIQUE, color INTEGER NOT NULL, pos INTEGER NOT NULL)")
        db.execSQL("CREATE TABLE history(id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT NOT NULL, url TEXT NOT NULL UNIQUE, visited_at INTEGER NOT NULL, visits INTEGER NOT NULL)")
        db.execSQL("CREATE INDEX history_time ON history(visited_at DESC)")
        db.execSQL("CREATE TABLE downloads(dm_id INTEGER PRIMARY KEY, name TEXT NOT NULL, url TEXT NOT NULL, at INTEGER NOT NULL)")
        val defaults = listOf(
            Triple("Википедия", "https://ru.wikipedia.org", 0xFF546E7A.toInt()),
            Triple("Хабр", "https://habr.com/ru", 0xFF0288D1.toInt()),
            Triple("4PDA", "https://4pda.to", 0xFF2E7D32.toInt()),
            Triple("GitHub", "https://github.com", 0xFF37474F.toInt()),
            Triple("Яндекс Погода", "https://yandex.ru/pogoda", 0xFFF9A825.toInt()),
            Triple("Reddit", "https://old.reddit.com", 0xFFE64A19.toInt()),
            Triple("OpenStreetMap", "https://www.openstreetmap.org", 0xFF00897B.toInt()),
            Triple("Internet Archive", "https://archive.org", 0xFF3949AB.toInt()),
        )
        defaults.forEachIndexed { i, (t, u, c) ->
            db.insert("bookmarks", null, ContentValues().apply {
                put("title", t); put("url", u); put("color", c); put("pos", i)
            })
        }
    }

    override fun onUpgrade(db: SQLiteDatabase, oldVersion: Int, newVersion: Int) {}

    // ───── закладки ─────

    fun bookmarks(): List<Bookmark> {
        val out = ArrayList<Bookmark>()
        readableDatabase.rawQuery("SELECT id,title,url,color FROM bookmarks ORDER BY pos, id", null).use { c ->
            while (c.moveToNext()) out += Bookmark(c.getLong(0), c.getString(1), c.getString(2), c.getInt(3))
        }
        return out
    }

    fun isBookmarked(url: String): Boolean =
        readableDatabase.rawQuery("SELECT 1 FROM bookmarks WHERE url=?", arrayOf(url)).use { it.moveToFirst() }

    fun addBookmark(title: String, url: String, color: Int = colorFor(url)): Boolean {
        val pos = readableDatabase.rawQuery("SELECT COALESCE(MAX(pos),-1)+1 FROM bookmarks", null).use { if (it.moveToFirst()) it.getInt(0) else 0 }
        val id = writableDatabase.insertWithOnConflict("bookmarks", null, ContentValues().apply {
            put("title", title.ifBlank { url }); put("url", url); put("color", color); put("pos", pos)
        }, SQLiteDatabase.CONFLICT_IGNORE)
        return id != -1L
    }

    fun removeBookmark(id: Long) { writableDatabase.delete("bookmarks", "id=?", arrayOf(id.toString())) }

    fun removeBookmarkByUrl(url: String) { writableDatabase.delete("bookmarks", "url=?", arrayOf(url)) }

    // ───── история ─────

    fun visit(title: String, url: String) {
        val now = System.currentTimeMillis()
        val db = writableDatabase
        val updated = db.compileStatement("UPDATE history SET visited_at=?, visits=visits+1, title=CASE WHEN ?<>'' THEN ? ELSE title END WHERE url=?").use {
            it.bindLong(1, now); it.bindString(2, title); it.bindString(3, title); it.bindString(4, url)
            it.executeUpdateDelete()
        }
        if (updated == 0) {
            db.insert("history", null, ContentValues().apply {
                put("title", title.ifBlank { url }); put("url", url); put("visited_at", now); put("visits", 1)
            })
            // История не должна расти бесконечно.
            db.execSQL("DELETE FROM history WHERE id NOT IN (SELECT id FROM history ORDER BY visited_at DESC LIMIT 5000)")
        }
    }

    fun updateHistoryTitle(url: String, title: String) {
        if (title.isBlank()) return
        writableDatabase.update("history", ContentValues().apply { put("title", title) }, "url=?", arrayOf(url))
    }

    fun history(filter: String = "", limit: Int = 300): List<HistoryItem> {
        val out = ArrayList<HistoryItem>()
        val like = "%" + filter.trim().replace("%", "").replace("_", "") + "%"
        val sql = if (filter.isBlank())
            "SELECT id,title,url,visited_at,visits FROM history ORDER BY visited_at DESC LIMIT $limit"
        else
            "SELECT id,title,url,visited_at,visits FROM history WHERE title LIKE ? OR url LIKE ? ORDER BY visited_at DESC LIMIT $limit"
        readableDatabase.rawQuery(sql, if (filter.isBlank()) null else arrayOf(like, like)).use { c ->
            while (c.moveToNext()) out += HistoryItem(c.getLong(0), c.getString(1), c.getString(2), c.getLong(3), c.getInt(4))
        }
        return out
    }

    fun deleteHistory(id: Long) { writableDatabase.delete("history", "id=?", arrayOf(id.toString())) }

    fun clearHistory() { writableDatabase.delete("history", null, null) }

    /** Строки для Rust-подсказок: вид \t заголовок \t url \t посещений \t время(сек). */
    fun suggestEntries(): String {
        val sb = StringBuilder()
        fun clean(s: String) = s.replace('\t', ' ').replace('\n', ' ').replace('\r', ' ')
        for (b in bookmarks()) sb.append("b\t").append(clean(b.title)).append('\t').append(clean(b.url)).append("\t0\t0\n")
        readableDatabase.rawQuery("SELECT title,url,visits,visited_at/1000 FROM history ORDER BY visited_at DESC LIMIT 1500", null).use { c ->
            while (c.moveToNext()) {
                sb.append("h\t").append(clean(c.getString(0))).append('\t').append(clean(c.getString(1)))
                    .append('\t').append(c.getInt(2)).append('\t').append(c.getLong(3)).append('\n')
            }
        }
        return sb.toString()
    }

    // ───── загрузки ─────

    fun addDownload(dmId: Long, name: String, url: String) {
        writableDatabase.insertWithOnConflict("downloads", null, ContentValues().apply {
            put("dm_id", dmId); put("name", name); put("url", url); put("at", System.currentTimeMillis())
        }, SQLiteDatabase.CONFLICT_REPLACE)
    }

    fun downloads(): List<DownloadRec> {
        val out = ArrayList<DownloadRec>()
        readableDatabase.rawQuery("SELECT dm_id,name,url,at FROM downloads ORDER BY at DESC LIMIT 200", null).use { c ->
            while (c.moveToNext()) out += DownloadRec(c.getLong(0), c.getString(1), c.getString(2), c.getLong(3))
        }
        return out
    }

    fun removeDownload(dmId: Long) { writableDatabase.delete("downloads", "dm_id=?", arrayOf(dmId.toString())) }

    fun clearDownloads() { writableDatabase.delete("downloads", null, null) }

    companion object {
        private val palette = intArrayOf(
            0xFF1E88E5.toInt(), 0xFF43A047.toInt(), 0xFFF4511E.toInt(), 0xFF8E24AA.toInt(),
            0xFF00897B.toInt(), 0xFFFB8C00.toInt(), 0xFF3949AB.toInt(), 0xFFD81B60.toInt(),
        )

        fun colorFor(url: String): Int = palette[(Math.abs(hostOf(url).hashCode())) % palette.size]

        fun hostOf(url: String): String = try {
            java.net.URI(url).host?.removePrefix("www.") ?: url
        } catch (_: Exception) {
            url
        }

        /** «Сегодня, 10:24» / «Вчера, 18:15» / «20 окт, 11:30». */
        fun whenText(ts: Long): String {
            val now = Calendar.getInstance()
            val t = Calendar.getInstance().apply { timeInMillis = ts }
            val hm = String.format("%02d:%02d", t.get(Calendar.HOUR_OF_DAY), t.get(Calendar.MINUTE))
            val dayDiff = (startOfDay(now) - startOfDay(t)) / 86_400_000L
            return when {
                dayDiff == 0L -> "Сегодня, $hm"
                dayDiff == 1L -> "Вчера, $hm"
                else -> {
                    val m = arrayOf("янв", "фев", "мар", "апр", "мая", "июн", "июл", "авг", "сен", "окт", "ноя", "дек")
                    "${t.get(Calendar.DAY_OF_MONTH)} ${m[t.get(Calendar.MONTH)]}, $hm"
                }
            }
        }

        private fun startOfDay(c: Calendar): Long = (c.clone() as Calendar).apply {
            set(Calendar.HOUR_OF_DAY, 0); set(Calendar.MINUTE, 0); set(Calendar.SECOND, 0); set(Calendar.MILLISECOND, 0)
        }.timeInMillis
    }
}
