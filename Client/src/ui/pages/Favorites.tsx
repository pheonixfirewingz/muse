import React from 'react';
import { HeartIcon } from '@heroicons/react/24/solid';

const Favorites: React.FC = () => {
    const favoriteTracks = [
        { id: 1, title: 'Starlight Symphony', artist: 'Aurora Moon', album: 'Midnight Dreams', duration: '3:45' },
        { id: 2, title: 'Electric Dreams', artist: 'The Neon Hearts', album: 'Electric Soul', duration: '4:12' },
        { id: 3, title: 'Sunset Boulevard', artist: 'Beach Boys Revival', album: 'Summer Vibes', duration: '3:58' },
        { id: 4, title: 'Midnight Jazz', artist: 'Miles Ahead', album: 'Jazz Nights', duration: '5:22' },
        { id: 5, title: 'Thunder Road', artist: 'Thunder Strike', album: 'Rock Anthems', duration: '4:05' },
        { id: 6, title: 'Moonlight Sonata', artist: 'Symphony Orchestra', album: 'Classical Dreams', duration: '6:15' },
        { id: 7, title: 'Urban Flow', artist: 'Urban Flow', album: 'Hip Hop Beats', duration: '3:33' },
        { id: 8, title: 'Starlight', artist: 'StarLight', album: 'Pop Sensation', duration: '3:28' },
        { id: 9, title: 'Dancing Queen', artist: 'Aurora Moon', album: 'Midnight Dreams', duration: '3:52' },
        { id: 10, title: 'Neon Nights', artist: 'The Neon Hearts', album: 'Electric Soul', duration: '4:18' },
    ];

    return (
        <div className="max-w-screen-2xl mx-auto">
            <div className="flex items-center gap-4 mb-8">
                <div className="w-16 h-16 bg-gradient-to-br from-rose-500 to-pink-600 rounded-lg flex items-center justify-center">
                    <HeartIcon className="w-8 h-8 text-white" />
                </div>
                <div>
                    <h2 className="text-4xl font-bold text-white">Favorites</h2>
                    <p className="text-white/60">{favoriteTracks.length} liked songs</p>
                </div>
            </div>
            
            <div className="bg-white/5 backdrop-blur-sm rounded-lg overflow-hidden">
                <div className="grid grid-cols-12 gap-4 px-6 py-3 border-b border-white/10 text-white/50 text-sm">
                    <div className="col-span-1">#</div>
                    <div className="col-span-5">Title</div>
                    <div className="col-span-3">Album</div>
                    <div className="col-span-2">Duration</div>
                    <div className="col-span-1"></div>
                </div>
                {favoriteTracks.map((track, index) => (
                    <div
                        key={track.id}
                        className="grid grid-cols-12 gap-4 px-6 py-4 hover:bg-white/5 transition-colors cursor-pointer group"
                    >
                        <div className="col-span-1 text-white/40 text-sm flex items-center">
                            {index + 1}
                        </div>
                        <div className="col-span-5 flex items-center">
                            <div>
                                <div className="text-white font-medium">{track.title}</div>
                                <div className="text-white/50 text-sm">{track.artist}</div>
                            </div>
                        </div>
                        <div className="col-span-3 text-white/50 text-sm flex items-center">
                            {track.album}
                        </div>
                        <div className="col-span-2 text-white/50 text-sm flex items-center">
                            {track.duration}
                        </div>
                        <div className="col-span-1 flex items-center justify-end">
                            <HeartIcon className="w-5 h-5 text-rose-500" />
                        </div>
                    </div>
                ))}
            </div>
        </div>
    );
};

export default Favorites;
