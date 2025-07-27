import { simulation, scenario, rampUsers, bodyString } from "@gatling.io/core";
import { http, ws } from "@gatling.io/http";

export default simulation(setUp => {
    const protocol = http
        .wsBaseUrl("ws://localhost:3000");

    // It verifies the received text message is a JSON with a 'data' field.
    const dataCheck = ws.checkTextMessage("checkData")
        .check(bodyString().exists());

    const wsScenario = scenario("WebSocket Location Test")
        .exec(ws("Connect WS").connect("/ws").await(2).on(dataCheck))
        .pause(5)
        .exec(ws("Close WS").close());

    setUp(wsScenario.injectOpen(rampUsers(5_000).during(120))).protocols(protocol);
});