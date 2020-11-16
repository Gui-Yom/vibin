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
import java.util.concurrent.locks.ReentrantLock
import javax.swing.event.MouseInputAdapter
import kotlin.concurrent.withLock
import kotlin.system.exitProcess

fun main(args: Array<String>) {

    val config: Config = Json.decodeFromString(File("config.vibin.json").readText())
    if (config.files.isEmpty()) {
        println("No registered media.")
        exitProcess(-1)
    }

    val frame = Frame("Vibin")
    frame.isResizable = false
    frame.isUndecorated = true
    frame.isAlwaysOnTop = true
    frame.size = Dimension(128, 128)
    frame.layout = BorderLayout()

    val factory = MediaPlayerFactory("--loop")
    val playerComponent = EmbeddedMediaPlayerComponent(factory, null, UnsupportedFullScreenStrategy(), InputEvents.DISABLE_NATIVE, null)
    val player = playerComponent.mediaPlayer()
    player.video().setScale(0.5f)

    val lock = ReentrantLock()
    val lockCondition = lock.newCondition()

    var mediaPointer = -1

    fun playNext() {
        val newPtr = ++mediaPointer
        val media = if (newPtr < config.files.size) config.files[newPtr] else {
            mediaPointer = 0
            config.files[0]
        }
        player.media().start(media.file, "--volume ${media.defaultVolume}")
    }

    val listener = object : MouseInputAdapter() {
        var location: Point? = null
        var pressed: MouseEvent? = null

        override fun mouseClicked(e: MouseEvent) {
            if (e.button == MouseEvent.BUTTON2) {
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
            player.submit {
                player.audio().setVolume(player.audio().volume() - e.wheelRotation * 5)
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
