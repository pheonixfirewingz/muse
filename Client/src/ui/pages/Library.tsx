import React, { useEffect, useState } from 'react';
import { apiService } from '../services/api';
import type { Song } from '../services/api';
import { usePlayback } from '../App';

const Library: React.FC = () => {
    const { setPlaylist } = usePlayback();
    const [songs, setSongs] = useState<Song[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [totalSongs, setTotalSongs] = useState(0);
    const [searchQuery, setSearchQuery] = useState('');

    useEffect(() => {
        fetchSongs();
    }, []);

    const fetchSongs = async () => {
        try {
            setLoading(true);
            setError(null);
            
            // Get total songs first
            const totalResponse = await apiService.getTotalSongs();
            if (totalResponse.success && totalResponse.data) {
                setTotalSongs(totalResponse.data.total);
                
                // Only fetch songs if there are any
                if (totalResponse.data.total > 0) {
                    // Fetch first batch of songs (0-49)
                    const endIndex = Math.min(49, totalResponse.data.total - 1);
                    console.log('Fetching songs from index 0 to', endIndex);
                    const songsResponse = await apiService.getSongs(0, endIndex);
                    console.log('Songs response:', songsResponse);
                    if (songsResponse.success && songsResponse.data) {
                        console.log('Setting songs:', songsResponse.data);
                        setSongs(songsResponse.data);
                    } else {
                        console.error('Failed to get songs or no data:', songsResponse);
                        setSongs([]);
                    }
                } else {
                    // No songs available
                    setSongs([]);
                }
            }
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to load songs');
            console.error('Error fetching songs:', err);
        } finally {
            setLoading(false);
        }
    };

    const handleSearch = async (query: string) => {
        setSearchQuery(query);
        
        if (!query.trim()) {
            // If search is cleared, reload original songs
            fetchSongs();
            return;
        }

        try {
            setLoading(true);
            setError(null);
            const response = await apiService.searchSongs(query);
            if (response.success && response.data) {
                setSongs(response.data);
            }
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Search failed');
            console.error('Error searching songs:', err);
        } finally {
            setLoading(false);
        }
    };

    if (loading && songs.length === 0) {
        return (
            <div className="max-w-screen-2xl mx-auto">
                <h2 className="text-4xl font-bold text-white mb-2">Music Library</h2>
                <p className="text-white/60 mb-8">Loading your music collection...</p>
                <div className="flex justify-center items-center h-64">
                    <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-white"></div>
                </div>
            </div>
        );
    }

    if (error) {
        return (
            <div className="max-w-screen-2xl mx-auto">
                <h2 className="text-4xl font-bold text-white mb-2">Music Library</h2>
                <p className="text-white/60 mb-8">Your complete music collection</p>
                <div className="bg-red-500/10 border border-red-500/20 rounded-lg p-4">
                    <p className="text-red-400">Error: {error}</p>
                    <button 
                        onClick={fetchSongs}
                        className="mt-4 px-4 py-2 bg-red-500 hover:bg-red-600 text-white rounded-md transition-colors"
                    >
                        Retry
                    </button>
                </div>
            </div>
        );
    }

    return (
        <div className="max-w-screen-2xl mx-auto">
            <h2 className="text-4xl font-bold text-white mb-2">Music Library</h2>
            <p className="text-white/60 mb-4">Your complete music collection</p>
            
            {/* Search Bar */}
            <div className="mb-6">
                <input
                    type="text"
                    placeholder="Search songs..."
                    value={searchQuery}
                    onChange={(e) => handleSearch(e.target.value)}
                    className="w-full max-w-md px-4 py-2 bg-white/5 backdrop-blur-sm border border-white/10 rounded-lg text-white placeholder-white/40 focus:outline-none focus:border-rose-500/50 transition-colors"
                />
            </div>

            {/* Stats */}
            <p className="text-white/40 text-sm mb-6">
                {searchQuery ? `Found ${songs.length} song${songs.length !== 1 ? 's' : ''}` : `${totalSongs} total song${totalSongs !== 1 ? 's' : ''}`}
            </p>
            
            {songs.length === 0 ? (
                <div className="text-center py-12">
                    <p className="text-white/60 text-lg">
                        {searchQuery ? 'No songs found matching your search.' : 'No songs in your library yet.'}
                    </p>
                </div>
            ) : (
                <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-6">
                    {songs.map((song, index) => (
                        <div
                            key={`${song.artist_name}-${song.name}-${index}`}
                            onClick={() => setPlaylist(songs, index)}
                            className="bg-white/5 backdrop-blur-sm rounded-lg p-4 hover:bg-white/10 transition-all cursor-pointer group relative"
                        >
                            {/* Play button overlay */}
                            <div className="absolute inset-0 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity bg-black/40 rounded-lg backdrop-blur-sm">
                                <div className="w-14 h-14 bg-rose-500 rounded-full flex items-center justify-center shadow-lg shadow-rose-500/50">
                                    <svg className="w-6 h-6 text-white ml-1" fill="currentColor" viewBox="0 0 24 24">
                                        <path d="M8 5v14l11-7z" />
                                    </svg>
                                </div>
                            </div>
                            <div className="aspect-square bg-gradient-to-br from-rose-500/20 to-purple-500/20 rounded-md mb-4 flex items-center justify-center overflow-hidden">
                                <div className="text-6xl">🎵</div>
                            </div>
                            <h3 className="text-white font-semibold text-sm mb-1 truncate" title={song.name}>
                                {song.name}
                            </h3>
                            <p className="text-white/50 text-xs truncate" title={song.artist_name}>
                                {song.artist_name}
                            </p>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
};

export default Library;
