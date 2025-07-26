import http from 'k6/http';
import { check } from 'k6';
import { Rate, Counter, Gauge } from 'k6/metrics';

// Custom metrics
export const sseConnections = new Counter('sse_connections_made');
export const sseMessages = new Counter('sse_messages_received');
export const sseDataReceived = new Counter('sse_data_bytes');
export const sseSuccessfulSessions = new Rate('sse_successful_sessions');

export const options = {
    scenarios: {
        sse_load_test: {
            executor: 'constant-vus',
            vus: 5000,
            duration: '20s',
        },
    },
    thresholds: {
        sse_successful_sessions: ['rate>0.95'],   // 95% successful sessions
    },
    // Don't fail the test on HTTP errors (timeouts are expected for SSE)
    noConnectionReuse: true,
};

export default function () {
    const params = {
        headers: {
            'Accept': 'text/event-stream',
            'Cache-Control': 'no-cache',
        },
        timeout: '5s', // 5-second SSE session (timeout expected)
    };

    sseConnections.add(1);

    try {
        const response = http.get('http://localhost:3000/sse', params);

        const hasData = response.body && response.body.length > 0;
        const isTimeout = response.error && response.error.includes('timeout');

        // A successful SSE session is one where we received data
        const sessionSuccess = hasData && (response.status === 200 || isTimeout);

        check(response, {
            'SSE session received data': () => hasData,
            'Session completed (timeout expected)': () => sessionSuccess,
        });

        if (hasData) {
            // Count data events and bytes received
            const dataEvents = (response.body.match(/^data: /gm) || []).length;
            sseMessages.add(dataEvents);
            sseDataReceived.add(response.body.length);

            // Mark as successful if we got data (even with timeout)
            sseSuccessfulSessions.add(1);
        } else {
            sseSuccessfulSessions.add(0);
        }

    } catch (error) {
        sseSuccessfulSessions.add(0);
    }

    // No sleep - immediately start next session (reconnect)
} 