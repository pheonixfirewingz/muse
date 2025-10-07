import React, { createContext, useContext, useState, useEffect } from 'react';
import type { ReactNode } from 'react';
import { apiService } from '../services/api';

interface AuthContextType {
  isAuthenticated: boolean;
  username: string | null;
  login: (username: string, token: string) => void;
  logout: () => void;
  checkAuth: () => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export const AuthProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [username, setUsername] = useState<string | null>(null);

  const checkAuth = () => {
    const authenticated = apiService.isAuthenticated();
    setIsAuthenticated(authenticated);
    
    if (authenticated) {
      // Fetch user info
      apiService.getUserInfo()
        .then(response => {
          if (response.success && response.data) {
            setUsername(response.data.username);
          }
        })
        .catch(() => {
          // If user info fails, clear auth
          logout();
        });
    } else {
      setUsername(null);
    }
  };

  const login = (user: string, token: string) => {
    apiService.setToken(token);
    setIsAuthenticated(true);
    setUsername(user);
  };

  const logout = () => {
    apiService.logout();
    setIsAuthenticated(false);
    setUsername(null);
  };

  useEffect(() => {
    checkAuth();
  }, []);

  return (
    <AuthContext.Provider value={{ isAuthenticated, username, login, logout, checkAuth }}>
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
};
