package marais.vibin

import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json
import uk.co.caprica.vlcj.factory.MediaPlayerFactory
import uk.co.caprica.vlcj.player.base.MediaPlayer
import uk.co.caprica.vlcj.player.base.MediaPlayerEventAdapter
import uk.co.caprica.vlcj.player.component.EmbeddedMediaPlayerComponent
import uk.co.caprica.vlcj.player.component.InputEvents
import uk.co.caprica.vlcj.player.embedded.fullscreen.unsupported.UnsupportedFullScreenStrategy
import java.awt.*
import java.awt.event.MouseEvent
import java.awt.event.MouseWheelEvent
import java.io.File
import java.net.URLConnection
import java.util.concurrent.locks.ReentrantLock
import javax.swing.event.MouseInputAdapter
import kotlin.concurrent.withLock
import kotlin.system.exitProcess

fun main() {

    val config: Config = Json.decodeFromString(File("config.vibin.json").readText())

    val mediaDir = File(config.player_settings.mediaDir)
    if (!mediaDir.exists() || !mediaDir.isDirectory || mediaDir.list() == null) {
        println("Wrong media directory ${config.player_settings.mediaDir}.")
        exitProcess(-1)
    }

    val mediaFiles = mediaDir.listFiles { file, name ->
        URLConnection.guessContentTypeFromName(name).startsWith("video")
    }
    if (mediaFiles == null || mediaFiles.isEmpty()) {
        println("No playable video found in ${config.player_settings.mediaDir}")
        exitProcess(-1)
    }

    val frame = Frame("Vibin")
    frame.isResizable = false
    frame.isUndecorated = true
    frame.isAlwaysOnTop = config.player_settings.alwaysOnTop
    frame.size = Dimension(128, 128)
    frame.layout = BorderLayout()
    frame.opacity = config.player_settings.opacity

    val factory = MediaPlayerFactory()
    val playerComponent =
        EmbeddedMediaPlayerComponent(factory, null, UnsupportedFullScreenStrategy(), InputEvents.DISABLE_NATIVE, null)
    val player = playerComponent.mediaPlayer()
    player.video().setScale(0.5f)

    val lock = ReentrantLock()
    val lockCondition = lock.newCondition()

    var mediaPointer = -1

    fun playNext() {
        val newPtr = ++mediaPointer
        val media = if (newPtr < mediaFiles.size) mediaFiles[newPtr] else {
            mediaPointer = 0
            mediaFiles[0]
        }

        val options = config.getMediaOptions(media.name) ?: MediaOptionsEntry("")
        player.media().start(config.player_settings.mediaDir + media, "--volume ${options.defaultVolume}")
    }

    val listener = object : MouseInputAdapter() {
        var location: Point? = null
        var pressed: MouseEvent? = null

        override fun mouseClicked(e: MouseEvent) {
            if (e.button == MouseEvent.BUTTON1 && e.isControlDown) {
                if (player.status().isPlaying) player.controls().pause() else player.controls().play()
            } else if (e.button == MouseEvent.BUTTON2) {
                lock.withLock { lockCondition.signalAll() }
            } else if (e.button == MouseEvent.BUTTON3) {
                player.controls().stop()
                playNext()
            }
        }

        override fun mousePressed(e: MouseEvent) {
            pressed = e
        }

        override fun mouseDragged(e: MouseEvent) {
            val component: Component = frame
            location = component.getLocation(location)
            val x: Int = location!!.x - pressed!!.x + e.x
            val y: Int = location!!.y - pressed!!.y + e.y
            component.setLocation(x, y)
        }

        override fun mouseWheelMoved(e: MouseWheelEvent) {
            if (e.isControlDown) {
                frame.opacity = (frame.opacity - e.wheelRotation * 0.1f).coerceIn(config.player_settings.minOpacity, 1f)
            } else {
                player.submit {
                    player.audio().setVolume(player.audio().volume() - e.wheelRotation * 5)
                }
                println(player.audio().volume())
            }
        }
    }
    playerComponent.videoSurfaceComponent().addMouseListener(listener)
    playerComponent.videoSurfaceComponent().addMouseMotionListener(listener)
    playerComponent.videoSurfaceComponent().addMouseWheelListener(listener)
    frame.add(playerComponent, BorderLayout.CENTER)
    frame.isVisible = true

    player.events().addMediaPlayerEventListener(object : MediaPlayerEventAdapter() {
        override fun finished(mediaPlayer: MediaPlayer?) {
            player.submit {
                playNext()
            }
        }
    })

    playNext()

    // Wait for window close event
    lock.withLock { lockCondition.await() }

    player.controls().stop()
    playerComponent.release()
    factory.release()
    frame.dispose()
}