package marais.vibin

import kotlinx.serialization.Serializable

@Serializable
data class Config(val files: Array<ConfigEntry>) {
    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (javaClass != other?.javaClass) return false

        other as Config

        if (!files.contentEquals(other.files)) return false

        return true
    }

    override fun hashCode(): Int {
        return files.contentHashCode()
    }
}

@Serializable
data class ConfigEntry(val file: String, val scale: Float = 0.5f, val defaultVolume: Int = 80)
