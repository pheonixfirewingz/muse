import { browser } from '$app/environment';

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

export interface AuthResponseData
{
	token: string;
	is_admin: boolean;
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
	errors?: Record<string, string>;
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

export interface Artist
{
	id: string;
	name: string;
}

class ApiService
{
	private token: string | null = null;

	constructor()
	{
		// Only access localStorage in the browser
		this.token = browser ? localStorage.getItem('auth_token') : null;
	}

	setToken(token: string | null)
	{
		this.token = token;
		// Only access localStorage in the browser
		if (browser) {
			if (token) localStorage.setItem('auth_token', token);
			else localStorage.removeItem('auth_token');
		}
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
			const errorData = await response.json().catch(() => ({ message: 'Request failed', errors: {} }));
			// Throw the entire error object as JSON string so we can parse it in the component
			throw new Error(JSON.stringify(errorData));
		}
		return response.json();
	}

	async login(credentials: LoginRequest): Promise<ApiResponse<AuthResponseData>>
	{
		const response = await this.request<ApiResponse<AuthResponseData>>('/login', {
			method: 'POST',
			body: JSON.stringify(credentials),
		});

		if (response.success && response.data?.token) this.setToken(response.data.token);
		return response;
	}

	async register(userData: RegisterRequest): Promise<ApiResponse<AuthResponseData>>
	{
		const response = await this.request<ApiResponse<AuthResponseData>>('/register', {
			method: 'POST',
			body: JSON.stringify(userData),
		});

		if (response.success && response.data?.token) this.setToken(response.data.token);
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

	async getSongs(): Promise<ApiResponse<Song[]>>
	{
		return this.request<ApiResponse<Song[]>>(`/songs`);
	}

	getSongCoverUrl(artistName: string, songName: string): string
	{
		return `${API_BASE_URL}/songs/cover?artist_name=${encodeURIComponent(artistName)}&name=${encodeURIComponent(songName)}`;
	}

	getStreamUrl(artistName: string, songName: string, format: string = 'mp3'): string
	{
		return `${API_BASE_URL}/stream?artist=${encodeURIComponent(artistName)}&name=${encodeURIComponent(songName)}&format=${format}`;
	}

	async getArtists(): Promise<ApiResponse<Artist[]>>
	{
		return this.request<ApiResponse<Artist[]>>(`/artists`);
	}
}

export const apiService = new ApiService();