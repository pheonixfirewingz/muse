import React, { useState, useRef, useEffect } from 'react';
import { Slider } from '@heroui/react';
import { 
    PlayIcon, 
    PauseIcon, 
    BackwardIcon, 
    ForwardIcon,
    SpeakerWaveIcon,
    ArrowPathIcon,
    ArrowsRightLeftIcon,
    QueueListIcon
} from '@heroicons/react/24/solid';
import { SpeakerXMarkIcon } from '@heroicons/react/24/outline';
import { apiService } from '../services/api';

interface MusicPlayerProps {
    currentTrack?: {
        name: string;
        artist_name: string;
    } | null;
    onNext?: () => void;
    onPrevious?: () => void;
}

const MusicPlayer: React.FC<MusicPlayerProps> = ({ currentTrack, onNext, onPrevious }) => {
    const audioRef = useRef<HTMLAudioElement | null>(null);
    const [isPlaying, setIsPlaying] = useState(false);
    const [currentTime, setCurrentTime] = useState(0);
    const [duration, setDuration] = useState(0);
    const [volume, setVolume] = useState(75);
    const [isMuted, setIsMuted] = useState(false);
    const [isRepeat, setIsRepeat] = useState(false);
    const [isShuffle, setIsShuffle] = useState(false);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    // Initialize audio element
    useEffect(() => {
        const audio = new Audio();
        audioRef.current = audio;

        // Set up event listeners
        audio.addEventListener('loadedmetadata', () => {
            setDuration(Math.floor(audio.duration));
            setIsLoading(false);
        });

        audio.addEventListener('timeupdate', () => {
            setCurrentTime(Math.floor(audio.currentTime));
        });

        audio.addEventListener('ended', () => {
            if (isRepeat) {
                audio.currentTime = 0;
                audio.play();
            } else {
                setIsPlaying(false);
                if (onNext) {
                    onNext();
                }
            }
        });

        audio.addEventListener('error', (e) => {
            console.error('Audio error:', e);
            setError('Failed to load audio');
            setIsLoading(false);
            setIsPlaying(false);
        });

        audio.addEventListener('canplay', () => {
            setError(null);
        });

        return () => {
            audio.pause();
            audio.src = '';
            audio.remove();
        };
    }, [isRepeat, onNext]);

    // Load new track when currentTrack changes
    useEffect(() => {
        if (!audioRef.current) return;

        const audio = audioRef.current;
        let blobUrl: string | null = null;

        // If no track is selected, clear the player
        if (!currentTrack) {
            audio.pause();
            if (audio.src) {
                // Clean up blob URL if it exists
                if (audio.src.startsWith('blob:')) {
                    URL.revokeObjectURL(audio.src);
                }
                audio.src = '';
            }
            setIsPlaying(false);
            setCurrentTime(0);
            setDuration(0);
            setError(null);
            setIsLoading(false);
            return;
        }

        setIsLoading(true);
        setError(null);
        
        // Build the stream URL with authentication
        const streamUrl = apiService.getStreamUrl(currentTrack.artist_name, currentTrack.name);
        const token = apiService.getToken();
        
        if (!token) {
            setError('Not authenticated');
            setIsLoading(false);
            return;
        }

        // Clean up previous blob URL if it exists
        if (audio.src && audio.src.startsWith('blob:')) {
            URL.revokeObjectURL(audio.src);
        }

        // We need to use fetch to add the auth header, then create a blob URL
        // This is required because HTML5 Audio doesn't support custom headers
        fetch(streamUrl, {
            headers: {
                'Authorization': `Bearer ${token}`
            }
        })
            .then(response => {
                if (!response.ok) {
                    if (response.status === 401) {
                        throw new Error('Authentication expired. Please log in again.');
                    }
                    throw new Error(`Failed to fetch audio (${response.status})`);
                }
                return response.blob();
            })
            .then(blob => {
                blobUrl = URL.createObjectURL(blob);
                audio.src = blobUrl;
                audio.load();
                
                // Auto-play if we were playing before
                if (isPlaying) {
                    audio.play().catch(err => {
                        console.error('Autoplay failed:', err);
                        setIsPlaying(false);
                    });
                }
            })
            .catch(err => {
                console.error('Error loading track:', err);
                setError(err.message || 'Failed to load track');
                setIsLoading(false);
            });

        // Cleanup function to revoke blob URL when component unmounts or track changes
        return () => {
            if (blobUrl) {
                URL.revokeObjectURL(blobUrl);
            }
        };
    }, [currentTrack]);

    // Update volume when volume state changes
    useEffect(() => {
        if (audioRef.current) {
            audioRef.current.volume = isMuted ? 0 : volume / 100;
        }
    }, [volume, isMuted]);

    const togglePlay = () => {
        if (!audioRef.current || !currentTrack) return;

        if (isPlaying) {
            audioRef.current.pause();
            setIsPlaying(false);
        } else {
            audioRef.current.play()
                .then(() => setIsPlaying(true))
                .catch(err => {
                    console.error('Play failed:', err);
                    setError('Failed to play');
                });
        }
    };

    const handleSeek = (value: number | number[]) => {
        const seekTime = Array.isArray(value) ? value[0] : value;
        if (audioRef.current) {
            audioRef.current.currentTime = seekTime;
            setCurrentTime(seekTime);
        }
    };

    const handleNext = () => {
        if (onNext) {
            onNext();
        }
    };

    const handlePrevious = () => {
        if (audioRef.current && currentTime > 3) {
            // If more than 3 seconds in, restart the song
            audioRef.current.currentTime = 0;
        } else if (onPrevious) {
            // Otherwise go to previous track
            onPrevious();
        }
    };

    const toggleMute = () => setIsMuted(!isMuted);
    const toggleRepeat = () => setIsRepeat(!isRepeat);
    const toggleShuffle = () => setIsShuffle(!isShuffle);

    const formatTime = (seconds: number) => {
        const mins = Math.floor(seconds / 60);
        const secs = seconds % 60;
        return `${mins}:${secs.toString().padStart(2, '0')}`;
    };

    return (
        <div className="w-full bg-gradient-to-t from-black/60 to-black/40 backdrop-blur-xl border-t border-white/10">
            {/* Progress bar */}
            <div className="group px-4 pt-2">
                <Slider
                    size="sm"
                    value={currentTime}
                    maxValue={duration || 100}
                    onChange={handleSeek}
                    isDisabled={!currentTrack || isLoading}
                    className="w-full"
                    classNames={{
                        track: "bg-white/20 group-hover:bg-white/30",
                        filler: "bg-rose-500",
                        thumb: "bg-white shadow-lg opacity-0 group-hover:opacity-100 transition-opacity"
                    }}
                />
            </div>

            {/* Main player controls */}
            <div className="px-6 pb-3">
                <div className="max-w-screen-2xl mx-auto flex items-center justify-between gap-4">
                    {/* Left: Track info with album art */}
                    <div className="flex items-center gap-3 flex-1 min-w-0">
                        <div className="w-14 h-14 bg-gradient-to-br from-rose-500/30 to-purple-500/30 rounded-md flex items-center justify-center flex-shrink-0">
                            <span className="text-2xl">🎵</span>
                        </div>
                        <div className="min-w-0 flex-1">
                            <p className="text-sm font-semibold text-white truncate">
                                {currentTrack?.name || 'No track selected'}
                            </p>
                            <p className="text-xs text-white/60 truncate">
                                {currentTrack?.artist_name || 'Select a song to play'}
                            </p>
                            {error && (
                                <p className="text-xs text-red-400 truncate">{error}</p>
                            )}
                            {isLoading && (
                                <p className="text-xs text-rose-400 truncate">Loading...</p>
                            )}
                        </div>
                        <button className="text-white/60 hover:text-rose-500 transition-colors p-2">
                            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z" />
                            </svg>
                        </button>
                    </div>

                    {/* Center: Playback controls */}
                    <div className="flex flex-col items-center gap-2 flex-shrink-0">
                        <div className="flex items-center gap-4">
                            {/* Shuffle */}
                            <button
                                onClick={toggleShuffle}
                                className={`p-2 rounded-full transition-all ${
                                    isShuffle 
                                        ? 'text-rose-500 bg-rose-500/20' 
                                        : 'text-white/60 hover:text-white hover:bg-white/10'
                                }`}
                            >
                                <ArrowsRightLeftIcon className="w-4 h-4" />
                            </button>

                            {/* Previous */}
                            <button 
                                onClick={handlePrevious}
                                disabled={!currentTrack}
                                className="text-white/80 hover:text-white transition-colors p-2 disabled:opacity-30 disabled:cursor-not-allowed"
                            >
                                <BackwardIcon className="w-5 h-5" />
                            </button>

                            {/* Play/Pause */}
                            <button
                                onClick={togglePlay}
                                disabled={!currentTrack || isLoading}
                                className="w-10 h-10 bg-rose-500 hover:bg-rose-600 rounded-full flex items-center justify-center transition-all hover:scale-105 shadow-lg shadow-rose-500/30 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:scale-100"
                            >
                                {isLoading ? (
                                    <div className="w-5 h-5 border-2 border-white border-t-transparent rounded-full animate-spin" />
                                ) : isPlaying ? (
                                    <PauseIcon className="w-5 h-5 text-white" />
                                ) : (
                                    <PlayIcon className="w-5 h-5 text-white ml-0.5" />
                                )}
                            </button>

                            {/* Next */}
                            <button 
                                onClick={handleNext}
                                disabled={!currentTrack}
                                className="text-white/80 hover:text-white transition-colors p-2 disabled:opacity-30 disabled:cursor-not-allowed"
                            >
                                <ForwardIcon className="w-5 h-5" />
                            </button>

                            {/* Repeat */}
                            <button
                                onClick={toggleRepeat}
                                className={`p-2 rounded-full transition-all ${
                                    isRepeat 
                                        ? 'text-rose-500 bg-rose-500/20' 
                                        : 'text-white/60 hover:text-white hover:bg-white/10'
                                }`}
                            >
                                <ArrowPathIcon className="w-4 h-4" />
                            </button>
                        </div>

                        {/* Time display */}
                        <div className="flex items-center gap-2 text-xs text-white/60">
                            <span>{formatTime(currentTime)}</span>
                            <span>/</span>
                            <span>{formatTime(duration)}</span>
                        </div>
                    </div>

                    {/* Right: Volume and queue */}
                    <div className="flex items-center gap-3 flex-1 justify-end">
                        <button className="text-white/60 hover:text-white transition-colors p-2">
                            <QueueListIcon className="w-5 h-5" />
                        </button>
                        
                        <div className="flex items-center gap-2 w-32">
                            <button onClick={toggleMute} className="text-white/60 hover:text-white transition-colors">
                                {isMuted || volume === 0 ? (
                                    <SpeakerXMarkIcon className="w-5 h-5" />
                                ) : (
                                    <SpeakerWaveIcon className="w-5 h-5" />
                                )}
                            </button>
                            <Slider
                                size="sm"
                                value={isMuted ? 0 : volume}
                                maxValue={100}
                                onChange={(value) => {
                                    setVolume(Array.isArray(value) ? value[0] : value);
                                    if (isMuted) setIsMuted(false);
                                }}
                                className="flex-1"
                                classNames={{
                                    track: "bg-white/20",
                                    filler: "bg-white/60"
                                }}
                            />
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default MusicPlayer;
