import { simulation, scenario, jsonPath, rampUsers } from "@gatling.io/core";
import { http, sse } from "@gatling.io/http";

export default simulation(setUp => {
  const protocol = http
    .baseUrl("http://localhost:3000")
    .acceptHeader("text/event-stream");

  // It verifies the received text message is a JSON with a 'data' field.
  const dataCheck = sse.checkMessage("checkData")
    .check(jsonPath("$.data").exists());

  const sseScenario = scenario("SSE Location Test")
    .exec(sse("Open stream").get("/sse").await(2).on(dataCheck))
    .pause(5)
    .exec(sse("Close stream").close())

  setUp(sseScenario.injectOpen(rampUsers(5_000).during(120))).protocols(protocol);
});