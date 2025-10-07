import React, { useEffect, useState } from 'react';
import { apiService, type Playlist } from '../services/api';

const Playlists: React.FC = () => {
    const [privatePlaylists, setPrivatePlaylists] = useState<Playlist[]>([]);
    const [publicPlaylists, setPublicPlaylists] = useState<Playlist[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        const fetchPlaylists = async () => {
            try {
                setLoading(true);
                setError(null);

                // Fetch counts first
                const [privateCountRes, publicCountRes] = await Promise.all([
                    apiService.getPrivatePlaylistCount(),
                    apiService.getPublicPlaylistCount()
                ]);

                const privateTotal = privateCountRes.data?.total || 0;
                const publicTotal = publicCountRes.data?.total || 0;

                // Fetch playlists if there are any
                const fetchPromises = [];
                
                if (privateTotal > 0) {
                    fetchPromises.push(
                        apiService.getPrivatePlaylists(0, Math.min(privateTotal - 1, 49))
                    );
                } else {
                    fetchPromises.push(Promise.resolve({ success: true, message: '', data: [] }));
                }

                if (publicTotal > 0) {
                    fetchPromises.push(
                        apiService.getPublicPlaylists(0, Math.min(publicTotal - 1, 49))
                    );
                } else {
                    fetchPromises.push(Promise.resolve({ success: true, message: '', data: [] }));
                }

                const [privateRes, publicRes] = await Promise.all(fetchPromises);

                setPrivatePlaylists(privateRes.data || []);
                setPublicPlaylists(publicRes.data || []);
            } catch (err) {
                setError(err instanceof Error ? err.message : 'Failed to fetch playlists');
            } finally {
                setLoading(false);
            }
        };

        fetchPlaylists();
    }, []);

    if (loading) {
        return (
            <div className="max-w-screen-2xl mx-auto flex items-center justify-center h-96">
                <p className="text-white/60 text-lg">Loading playlists...</p>
            </div>
        );
    }

    if (error) {
        return (
            <div className="max-w-screen-2xl mx-auto flex items-center justify-center h-96">
                <p className="text-rose-500 text-lg">Error: {error}</p>
            </div>
        );
    }

    const renderPlaylist = (playlist: Playlist, index: number) => {
        const icons = ['🎧', '💪', '📚', '🎉', '🚗', '☕', '🌃', '📼', '🎵', '🎸', '🎹', '🎺'];
        const icon = icons[index % icons.length];

        return (
            <div
                key={`${playlist.name}-${playlist.owner}`}
                className="bg-white/5 backdrop-blur-sm rounded-lg p-5 hover:bg-white/10 transition-all cursor-pointer group"
            >
                <div className="aspect-square bg-gradient-to-br from-rose-500/30 to-orange-500/20 rounded-md mb-4 flex items-center justify-center text-7xl">
                    {icon}
                </div>
                <h3 className="text-white font-bold text-base mb-1 truncate">
                    {playlist.name}
                </h3>
                <p className="text-white/50 text-xs mb-2">
                    {playlist.isPublic ? `By ${playlist.owner}` : 'Private'}
                </p>
                <p className="text-rose-500 text-xs font-medium">
                    {playlist.isPublic ? '🌐 Public' : '🔒 Private'}
                </p>
            </div>
        );
    };

    return (
        <div className="max-w-screen-2xl mx-auto">
            {/* Private Playlists Section */}
            {privatePlaylists.length > 0 && (
                <div className="mb-12">
                    <h2 className="text-4xl font-bold text-white mb-2">Your Private Playlists</h2>
                    <p className="text-white/60 mb-8">Your personal collections</p>
                    
                    <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6">
                        {privatePlaylists.map((playlist, index) => renderPlaylist(playlist, index))}
                    </div>
                </div>
            )}

            {/* Public Playlists Section */}
            {publicPlaylists.length > 0 && (
                <div>
                    <h2 className="text-4xl font-bold text-white mb-2">Public Playlists</h2>
                    <p className="text-white/60 mb-8">Shared by the community</p>
                    
                    <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6">
                        {publicPlaylists.map((playlist, index) => renderPlaylist(playlist, index))}
                    </div>
                </div>
            )}

            {/* Empty State */}
            {privatePlaylists.length === 0 && publicPlaylists.length === 0 && (
                <div className="flex flex-col items-center justify-center h-96">
                    <p className="text-white/60 text-lg mb-4">No playlists found</p>
                    <p className="text-white/40 text-sm">Create your first playlist to get started</p>
                </div>
            )}
        </div>
    );
};

export default Playlists;
