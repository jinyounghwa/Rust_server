// ===== API Configuration =====
const API_BASE_URL = window.location.origin;
const API_ENDPOINTS = {
    register: `${API_BASE_URL}/auth/register`,
    login: `${API_BASE_URL}/auth/login`,
    refresh: `${API_BASE_URL}/auth/refresh`,
    me: `${API_BASE_URL}/auth/me`,
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

// ===== Button Loading State =====
function setButtonLoading(button, isLoading) {
    const btnText = button.querySelector('.btn-text');
    const btnLoader = button.querySelector('.btn-loader');
    
    if (isLoading) {
        button.disabled = true;
        btnText.style.display = 'none';
        btnLoader.style.display = 'block';
    } else {
        button.disabled = false;
        btnText.style.display = 'block';
        btnLoader.style.display = 'none';
    }
}

// ===== Form Validation =====
function validateEmail(email) {
    const re = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return re.test(email);
}

function validatePassword(password) {
    // 8자 이상, 대문자, 소문자, 숫자 포함
    const hasMinLength = password.length >= 8;
    const hasUpperCase = /[A-Z]/.test(password);
    const hasLowerCase = /[a-z]/.test(password);
    const hasNumber = /\d/.test(password);
    
    return hasMinLength && hasUpperCase && hasLowerCase && hasNumber;
}

function validateName(name) {
    return name.trim().length >= 2;
}

// ===== API Calls =====
async function apiCall(endpoint, method = 'GET', data = null, useAuth = false) {
    const headers = {
        'Content-Type': 'application/json',
    };
    
    if (useAuth) {
        const token = TokenManager.getAccessToken();
        if (token) {
            headers['Authorization'] = `Bearer ${token}`;
        }
    }
    
    const options = {
        method,
        headers,
    };
    
    if (data && (method === 'POST' || method === 'PUT')) {
        options.body = JSON.stringify(data);
    }
    
    try {
        const response = await fetch(endpoint, options);
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

// ===== Login Handler =====
async function handleLogin(event) {
    event.preventDefault();
    
    const form = event.target;
    const email = form.email.value.trim();
    const password = form.password.value;
    const submitBtn = document.getElementById('login-btn');
    
    // Validation
    if (!validateEmail(email)) {
        AlertSystem.error('올바른 이메일 주소를 입력해주세요.');
        return;
    }
    
    if (!password) {
        AlertSystem.error('비밀번호를 입력해주세요.');
        return;
    }
    
    setButtonLoading(submitBtn, true);
    
    try {
        const data = await apiCall(API_ENDPOINTS.login, 'POST', {
            email,
            password
        });
        
        // Save tokens
        TokenManager.setTokens(data.access_token, data.refresh_token);
        
        AlertSystem.success('로그인 성공! 대시보드로 이동합니다...');
        
        setTimeout(() => {
            window.location.href = '/dashboard.html';
        }, 1000);
        
    } catch (error) {
        AlertSystem.error(error.message || '로그인에 실패했습니다. 이메일과 비밀번호를 확인해주세요.');
    } finally {
        setButtonLoading(submitBtn, false);
    }
}

// ===== Register Handler =====
async function handleRegister(event) {
    event.preventDefault();
    
    const form = event.target;
    const name = form.name.value.trim();
    const email = form.email.value.trim();
    const password = form.password.value;
    const submitBtn = document.getElementById('register-btn');
    
    // Validation
    if (!validateName(name)) {
        AlertSystem.error('이름은 2자 이상이어야 합니다.');
        return;
    }
    
    if (!validateEmail(email)) {
        AlertSystem.error('올바른 이메일 주소를 입력해주세요.');
        return;
    }
    
    if (!validatePassword(password)) {
        AlertSystem.error('비밀번호는 8자 이상이며, 대문자, 소문자, 숫자를 포함해야 합니다.');
        return;
    }
    
    setButtonLoading(submitBtn, true);
    
    try {
        const data = await apiCall(API_ENDPOINTS.register, 'POST', {
            name,
            email,
            password
        });
        
        // Save tokens
        TokenManager.setTokens(data.access_token, data.refresh_token);
        
        AlertSystem.success('회원가입 성공! 대시보드로 이동합니다...');
        
        setTimeout(() => {
            window.location.href = '/dashboard.html';
        }, 1000);
        
    } catch (error) {
        AlertSystem.error(error.message || '회원가입에 실패했습니다. 다시 시도해주세요.');
    } finally {
        setButtonLoading(submitBtn, false);
    }
}

// ===== Form Toggle =====
function setupFormToggle() {
    const showRegisterBtn = document.getElementById('show-register');
    const showLoginBtn = document.getElementById('show-login');
    const loginForm = document.getElementById('loginForm');
    const registerForm = document.getElementById('registerForm');
    
    showRegisterBtn.addEventListener('click', (e) => {
        e.preventDefault();
        loginForm.classList.remove('active');
        registerForm.classList.add('active');
        document.getElementById('alert-container').innerHTML = '';
    });
    
    showLoginBtn.addEventListener('click', (e) => {
        e.preventDefault();
        registerForm.classList.remove('active');
        loginForm.classList.add('active');
        document.getElementById('alert-container').innerHTML = '';
    });
}

// ===== Initialize =====
document.addEventListener('DOMContentLoaded', () => {
    // Check if already logged in
    if (TokenManager.hasValidToken()) {
        window.location.href = '/dashboard.html';
        return;
    }
    
    // Setup form handlers
    const loginForm = document.getElementById('login-form');
    const registerForm = document.getElementById('register-form');
    
    if (loginForm) {
        loginForm.addEventListener('submit', handleLogin);
    }
    
    if (registerForm) {
        registerForm.addEventListener('submit', handleRegister);
    }
    
    // Setup form toggle
    setupFormToggle();
});
