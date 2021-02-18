use swayipc_async::Node;

pub fn mock_firefox_node() -> Node {
    let payload = "{\"id\":6,\"name\":\"serde_json - Rust — Firefox Developer Edition\",\"type\":\"con\",\"border\":\"pixel\",\"current_border_width\":1,\"layout\":\"none\",\"percent\":1.0,\"rect\":{\"x\":0,\"y\":21,\"width\":1280,\"height\":779},\"window_rect\":{\"x\":0,\"y\":1,\"width\":1280,\"height\":779},\"deco_rect\":{\"x\":0,\"y\":0,\"width\":0,\"height\":0},\"geometry\":{\"x\":0,\"y\":0,\"width\":1280,\"height\":779},\"urgent\":false,\"focused\":true,\"focus\":[],\"nodes\":[],\"floating_nodes\":[],\"sticky\":false,\"representation\":null,\"fullscreen_mode\":0,\"app_id\":\"firefoxdeveloperedition\",\"pid\":901,\"window\":null,\"num\":null,\"window_properties\":null,\"marks\":[],\"inhibit_idle\":false,\"idle_inhibitors\":{\"application\":\"none\",\"user\":\"none\"},\"shell\":\"xdg_shell\"}";
    serde_json::from_slice(payload.as_bytes()).unwrap()
}
