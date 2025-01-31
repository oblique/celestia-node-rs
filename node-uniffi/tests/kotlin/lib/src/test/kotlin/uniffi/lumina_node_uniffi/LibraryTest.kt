package uniffi.lumina_node_uniffi

import kotlin.test.Test
import kotlin.test.assertTrue
import uniffi.lumina_node_uniffi.*
import uniffi.lumina_node.*

class LibraryTest {
    @Test fun someLibraryMethodReturnsTrue() {
        System.loadLibrary("lumina_node_uniffi")
        //System.load("/home/oblique/work/celestia/lumina/target/debug/liblumina_node_uniffi.so")
        var config = NodeConfig(null, Network.Mainnet, null, null, null, null, null)
        var node = LuminaNode(config)
    }
}
