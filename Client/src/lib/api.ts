const API_BASE_URL = 'http://127.0.0.1:8000/api';

export interface LoginRequest
{
  username: string;
  password: string;
}

export interface RegisterRequest
{
  username: string;
  email: string;
  password: string;
  confirm_password: string;
}

export interface AuthResponse
{
  success: boolean;
  message: string;
  token?: string;
  errors?: Record<string, string>;
}

export interface UserInfo
{
  username: string;
  email: string;
}

export interface ApiResponse<T>
{
  success: boolean;
  message: string;
  data?: T;
  error?: string;
}

export interface Playlist
{
  name: string;
  owner: string;
  isPublic: boolean;
}

export interface PlaylistTotal
{
  total: number;
}

export interface Song
{
  name: string;
  artist_name: string;
}

export interface SongTotal
{
  total: number;
}

class ApiService
{
  private token: string | null = null;

  constructor()
	{
    this.token = localStorage.getItem('auth_token');
  }

  setToken(token: string | null)
	{
    this.token = token;
    if (token) localStorage.setItem('auth_token', token);
    else localStorage.removeItem('auth_token');
  }

  getToken(): string | null
	{
    return this.token;
  }

  private async request<T>(endpoint: string, options: RequestInit = {}): Promise<T>
	{
    const url = `${API_BASE_URL}${endpoint}`;
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };

    if (this.token && !endpoint.includes('/login') && !endpoint.includes('/register'))
      headers['Authorization'] = `Bearer ${this.token}`;

    const response = await fetch(url, {
      ...options,
      headers,
    });

    if (!response.ok)
		{
      if (response.status === 401)
			{
        this.setToken(null);
        throw new Error('Unauthorized');
      }
      const errorData = await response.json().catch(() => ({ error: 'Request failed' }));
      throw new Error(errorData.error || `HTTP ${response.status}`);
    }
    return response.json();
  }

  async login(credentials: LoginRequest): Promise<AuthResponse> {
    const response = await this.request<AuthResponse>('/login', {
      method: 'POST',
      body: JSON.stringify(credentials),
    });

    if (response.success && response.token) this.setToken(response.token);
    return response;
  }

  async register(userData: RegisterRequest): Promise<AuthResponse>
	{
    const response = await this.request<AuthResponse>('/register', {
      method: 'POST',
      body: JSON.stringify(userData),
    });

    if (response.success && response.token) this.setToken(response.token);
    return response;
  }

  async getUserInfo(): Promise<ApiResponse<UserInfo>>
	{
    return this.request<ApiResponse<UserInfo>>('/user');
  }

  logout()
	{
    this.setToken(null);
  }

  isAuthenticated(): boolean
	{
    return this.token !== null;
  }

  async getPrivatePlaylists(indexStart: number, indexEnd: number): Promise<ApiResponse<Playlist[]>>
	{
    return this.request<ApiResponse<Playlist[]>>(`/playlists/private?index_start=${indexStart}&index_end=${indexEnd}`);
  }

  async getPublicPlaylists(indexStart: number, indexEnd: number): Promise<ApiResponse<Playlist[]>>
	{
    return this.request<ApiResponse<Playlist[]>>(`/playlists/public?index_start=${indexStart}&index_end=${indexEnd}`);
  }

  async getPrivatePlaylistCount(): Promise<ApiResponse<PlaylistTotal>>
	{
    return this.request<ApiResponse<PlaylistTotal>>('/playlists/private/total');
  }

  async getPublicPlaylistCount(): Promise<ApiResponse<PlaylistTotal>>
	{
    return this.request<ApiResponse<PlaylistTotal>>('/playlists/public/total');
  }

  async getSongs(indexStart: number, indexEnd: number): Promise<ApiResponse<Song[]>>
	{
    return this.request<ApiResponse<Song[]>>(`/songs?index_start=${indexStart}&index_end=${indexEnd}`);
  }

  async getTotalSongs(): Promise<ApiResponse<SongTotal>>
	{
    return this.request<ApiResponse<SongTotal>>('/songs/total');
  }

  async searchSongs(query: string): Promise<ApiResponse<Song[]>>
	{
    return this.request<ApiResponse<Song[]>>(`/songs/search?query=${encodeURIComponent(query)}`);
  }

  getSongCoverUrl(artistName: string, songName: string): string
	{
    return `${API_BASE_URL}/songs/cover?artist_name=${encodeURIComponent(artistName)}&name=${encodeURIComponent(songName)}`;
  }

  getStreamUrl(artistName: string, songName: string, format: string = 'mp3'): string
	{
    return `${API_BASE_URL}/stream?artist=${encodeURIComponent(artistName)}&name=${encodeURIComponent(songName)}&format=${format}`;
  }
}

export const apiService = new ApiService();
