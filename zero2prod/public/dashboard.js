// ===== API Configuration =====
const API_BASE_URL = window.location.origin;
const API_ENDPOINTS = {
    me: `${API_BASE_URL}/auth/me`,
    refresh: `${API_BASE_URL}/auth/refresh`,
};

// ===== Token Management =====
const TokenManager = {
    getAccessToken() {
        return localStorage.getItem('access_token');
    },
    
    getRefreshToken() {
        return localStorage.getItem('refresh_token');
    },
    
    setTokens(accessToken, refreshToken) {
        localStorage.setItem('access_token', accessToken);
        localStorage.setItem('refresh_token', refreshToken);
    },
    
    clearTokens() {
        localStorage.removeItem('access_token');
        localStorage.removeItem('refresh_token');
    },
    
    hasValidToken() {
        return !!this.getAccessToken();
    }
};

// ===== Alert System =====
const AlertSystem = {
    show(message, type = 'info') {
        const container = document.getElementById('alert-container');
        const alert = document.createElement('div');
        alert.className = `alert alert-${type}`;
        alert.textContent = message;
        
        container.innerHTML = '';
        container.appendChild(alert);
        
        setTimeout(() => {
            alert.style.opacity = '0';
            setTimeout(() => alert.remove(), 300);
        }, 5000);
    },
    
    success(message) {
        this.show(message, 'success');
    },
    
    error(message) {
        this.show(message, 'error');
    },
    
    info(message) {
        this.show(message, 'info');
    }
};

// ===== API Calls =====
async function apiCall(endpoint, method = 'GET', data = null) {
    const token = TokenManager.getAccessToken();
    
    const headers = {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${token}`,
    };
    
    const options = {
        method,
        headers,
    };
    
    if (data && (method === 'POST' || method === 'PUT')) {
        options.body = JSON.stringify(data);
    }
    
    try {
        const response = await fetch(endpoint, options);
        
        // Handle 401 - try to refresh token
        if (response.status === 401) {
            const refreshed = await refreshAccessToken();
            if (refreshed) {
                // Retry the original request
                return apiCall(endpoint, method, data);
            } else {
                throw new Error('인증이 만료되었습니다. 다시 로그인해주세요.');
            }
        }
        
        const responseData = await response.json();
        
        if (!response.ok) {
            throw new Error(responseData.message || responseData.error || '요청 처리 중 오류가 발생했습니다.');
        }
        
        return responseData;
    } catch (error) {
        console.error('API Error:', error);
        throw error;
    }
}

// ===== Token Refresh =====
async function refreshAccessToken() {
    const refreshToken = TokenManager.getRefreshToken();
    
    if (!refreshToken) {
        return false;
    }
    
    try {
        const response = await fetch(API_ENDPOINTS.refresh, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                refresh_token: refreshToken
            })
        });
        
        if (!response.ok) {
            return false;
        }
        
        const data = await response.json();
        TokenManager.setTokens(data.access_token, data.refresh_token);
        
        return true;
    } catch (error) {
        console.error('Token refresh failed:', error);
        return false;
    }
}

// ===== Load User Info =====
async function loadUserInfo() {
    const userInfoContainer = document.getElementById('user-info');
    
    try {
        const userData = await apiCall(API_ENDPOINTS.me);
        
        // Format date
        const createdDate = new Date(userData.created_at);
        const formattedDate = createdDate.toLocaleDateString('ko-KR', {
            year: 'numeric',
            month: 'long',
            day: 'numeric',
            hour: '2-digit',
            minute: '2-digit'
        });
        
        userInfoContainer.innerHTML = `
            <div class="user-detail">
                <div class="detail-row">
                    <span class="detail-label">사용자 ID</span>
                    <span class="detail-value">${userData.id}</span>
                </div>
                <div class="detail-row">
                    <span class="detail-label">이름</span>
                    <span class="detail-value">${userData.name}</span>
                </div>
                <div class="detail-row">
                    <span class="detail-label">이메일</span>
                    <span class="detail-value">${userData.email}</span>
                </div>
                <div class="detail-row">
                    <span class="detail-label">가입일</span>
                    <span class="detail-value">${formattedDate}</span>
                </div>
            </div>
        `;
        
    } catch (error) {
        userInfoContainer.innerHTML = `
            <div class="info-loading">
                <p style="color: var(--error-color);">사용자 정보를 불러올 수 없습니다.</p>
                <p style="font-size: 0.875rem; color: var(--gray-500);">${error.message}</p>
            </div>
        `;
        
        AlertSystem.error('사용자 정보를 불러오는데 실패했습니다.');
        
        // If authentication failed, redirect to login
        setTimeout(() => {
            logout();
        }, 2000);
    }
}

// ===== Logout =====
function logout() {
    TokenManager.clearTokens();
    AlertSystem.info('로그아웃되었습니다.');
    setTimeout(() => {
        window.location.href = '/';
    }, 1000);
}

// ===== Test API =====
async function testProtectedAPI() {
    const resultContainer = document.getElementById('api-result');
    const testBtn = document.getElementById('test-api-btn');
    
    testBtn.disabled = true;
    resultContainer.textContent = '요청 중...';
    
    try {
        const userData = await apiCall(API_ENDPOINTS.me);
        
        resultContainer.textContent = JSON.stringify(userData, null, 2);
        AlertSystem.success('API 호출 성공!');
        
    } catch (error) {
        resultContainer.textContent = `오류: ${error.message}`;
        AlertSystem.error('API 호출 실패');
    } finally {
        testBtn.disabled = false;
    }
}

// ===== Initialize =====
document.addEventListener('DOMContentLoaded', () => {
    // Check if user is logged in
    if (!TokenManager.hasValidToken()) {
        AlertSystem.error('로그인이 필요합니다.');
        setTimeout(() => {
            window.location.href = '/';
        }, 1000);
        return;
    }
    
    // Load user information
    loadUserInfo();
    
    // Setup logout button
    const logoutBtn = document.getElementById('logout-btn');
    if (logoutBtn) {
        logoutBtn.addEventListener('click', logout);
    }
    
    // Setup API test button
    const testApiBtn = document.getElementById('test-api-btn');
    if (testApiBtn) {
        testApiBtn.addEventListener('click', testProtectedAPI);
    }
});
