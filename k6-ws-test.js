import ws from 'k6/ws';
import { check } from 'k6';
import { Rate, Counter, Gauge } from 'k6/metrics';

// Custom metrics
export const wsConnections = new Counter('ws_connections_made');
export const wsMessages = new Counter('ws_messages_received');
export const wsDataReceived = new Counter('ws_data_bytes');
export const wsActiveConnections = new Gauge('ws_active_connections');
export const wsSuccessfulSessions = new Rate('ws_successful_sessions');

export const options = {
    scenarios: {
        ws_load_test: {
            executor: 'constant-vus',
            vus: 15000,
            duration: '20s',
        },
    },
    thresholds: {
        ws_successful_sessions: ['rate>0.95'],   // 95% successful sessions
    },
};

export default function () {
    const url = 'ws://localhost:3000/ws';
    let sessionSuccess = false;

    wsConnections.add(1);

    const response = ws.connect(url, {}, function (socket) {
        wsActiveConnections.add(1);

        socket.on('message', function message(data) {
            wsMessages.add(1);
            wsDataReceived.add(data.length);
            sessionSuccess = true;
        });

        socket.on('close', function close() {
            wsActiveConnections.add(-1);
        });

        socket.setTimeout(function () {
            socket.close();
        }, 20000);
    });

    check(response, {
        'WebSocket connection successful': (r) => r && r.status === 101,
    });

    wsSuccessfulSessions.add(sessionSuccess ? 1 : 0);
} 