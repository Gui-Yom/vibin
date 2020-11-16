package marais.vibin

import kotlinx.serialization.Serializable

@Serializable
data class Config(val mediaOptions: Array<MediaOptionsEntry>, val player_settings: PlayerSettings) {
    override fun equals(other: Any?): Boolean {

        if (this === other) return true
        if (javaClass != other?.javaClass) return false

        other as Config

        if (!mediaOptions.contentEquals(other.mediaOptions)) return false
        if (player_settings != other.player_settings) return false // fuck ça

        return true
    }

    override fun hashCode(): Int {
        return mediaOptions.contentHashCode() + player_settings.hashCode()
    }

    fun getMediaOptions(file : String) : MediaOptionsEntry? {

        for (entry in mediaOptions)
            if (entry.file == file) return entry;

        return null;
    }
}

@Serializable
data class MediaOptionsEntry(val file: String, val scale: Float = 0.5f, val defaultVolume: Int = 80)

@Serializable
data class PlayerSettings(val mediaDir : String = "./",
                          val alwaysOnTop : Boolean = true,
                          val opacity : Float = 1f,
                          val minOpacity : Float = 0.05f)