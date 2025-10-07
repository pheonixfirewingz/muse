import React, { useState, createContext, useContext } from 'react';
import { AuthProvider, useAuth } from './contexts/AuthContext';
import SideBar from "./components/sidebar.tsx";
import MusicPlayer from "./components/MusicPlayer.tsx";
import Library from "./pages/Library.tsx";
import Playlists from "./pages/Playlists.tsx";
import Favorites from "./pages/Favorites.tsx";
import Artists from "./pages/Artists.tsx";
import Profile from "./pages/Profile.tsx";
import Login from "./pages/Login.tsx";
import Register from "./pages/Register.tsx";
import type { Song } from './services/api';

interface PlaybackContextType {
    currentTrack: Song | null;
    playlist: Song[];
    currentIndex: number;
    setCurrentTrack: (track: Song) => void;
    setPlaylist: (songs: Song[], startIndex?: number) => void;
    playNext: () => void;
    playPrevious: () => void;
}

const PlaybackContext = createContext<PlaybackContextType | undefined>(undefined);

export const usePlayback = () => {
    const context = useContext(PlaybackContext);
    if (!context) {
        throw new Error('usePlayback must be used within PlaybackProvider');
    }
    return context;
};

const MainApp: React.FC = () => {
    const { isAuthenticated } = useAuth();
    const [currentTrack, setCurrentTrack] = useState<Song | null>(null);
    const [playlist, setPlaylistState] = useState<Song[]>([]);
    const [currentIndex, setCurrentIndex] = useState(0);
    const [currentPage, setCurrentPage] = useState<string>('library');
    const [authView, setAuthView] = useState<'login' | 'register'>('login');

    const setPlaylist = (songs: Song[], startIndex: number = 0) => {
        setPlaylistState(songs);
        setCurrentIndex(startIndex);
        if (songs.length > 0 && startIndex < songs.length) {
            setCurrentTrack(songs[startIndex]);
        }
    };

    const playNext = () => {
        if (playlist.length === 0) return;
        const nextIndex = (currentIndex + 1) % playlist.length;
        setCurrentIndex(nextIndex);
        setCurrentTrack(playlist[nextIndex]);
    };

    const playPrevious = () => {
        if (playlist.length === 0) return;
        const prevIndex = currentIndex === 0 ? playlist.length - 1 : currentIndex - 1;
        setCurrentIndex(prevIndex);
        setCurrentTrack(playlist[prevIndex]);
    };

    const handleSetCurrentTrack = (track: Song) => {
        setCurrentTrack(track);
        // If this track is in the current playlist, update the index
        const index = playlist.findIndex(
            s => s.name === track.name && s.artist_name === track.artist_name
        );
        if (index !== -1) {
            setCurrentIndex(index);
        } else {
            // If not in playlist, create a new playlist with just this track
            setPlaylistState([track]);
            setCurrentIndex(0);
        }
    };

    const playbackValue: PlaybackContextType = {
        currentTrack,
        playlist,
        currentIndex,
        setCurrentTrack: handleSetCurrentTrack,
        setPlaylist,
        playNext,
        playPrevious,
    };

    const renderPage = () => {
        switch (currentPage) {
            case 'library':
                return <Library />;
            case 'playlists':
                return <Playlists />;
            case 'favorites':
                return <Favorites />;
            case 'artists':
                return <Artists />;
            case 'profile':
                return <Profile />;
            default:
                return <Library />;
        }
    };

    if (!isAuthenticated) {
        if (authView === 'login') {
            return <Login onSwitchToRegister={() => setAuthView('register')} />;
        } else {
            return <Register onSwitchToLogin={() => setAuthView('login')} />;
        }
    }

    return (
        <PlaybackContext.Provider value={playbackValue}>
            <div className="flex h-screen bg-gradient-to-br from-black via-gray-900 to-black overflow-hidden">
                <SideBar currentPage={currentPage} onNavigate={setCurrentPage} />
                <div className="flex-1 flex flex-col overflow-hidden">
                    <div className="flex-1 overflow-y-auto p-8">
                        {renderPage()}
                    </div>
                    <MusicPlayer 
                        currentTrack={currentTrack} 
                        onNext={playNext}
                        onPrevious={playPrevious}
                    />
                </div>
            </div>
        </PlaybackContext.Provider>
    );
};

const App: React.FC = () => {
    return (
        <AuthProvider>
            <MainApp />
        </AuthProvider>
    );
};

export default App
