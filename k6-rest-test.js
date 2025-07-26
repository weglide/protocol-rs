import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

// Custom metrics
export const errorRate = new Rate('errors');

export const options = {
    stages: [
        { duration: '20s', target: 5000 },
    ],
    thresholds: {
        http_req_duration: ['p(95)<1000'], // 95% of requests must complete within 1s
        http_req_failed: ['rate<0.01'],    // Error rate must be less than 1%
        errors: ['rate<0.01'],
    },
};

export default function () {
    const response = http.get('http://localhost:3000/');

    const success = check(response, {
        'status is 200': (r) => r.status === 200,
        'response body contains data': (r) => {
            try {
                const data = JSON.parse(r.body);
                return Array.isArray(data) && data.length === 500;
            } catch (e) {
                return false;
            }
        },
        'response time < 500ms': (r) => r.timings.duration < 500,
    });

    errorRate.add(!success);

    // Small pause between requests
    sleep(0.5);
} 