import React from 'react';

const Artists: React.FC = () => {
    const artists = [
        { id: 1, name: 'Aurora Moon', followers: '2.5M', image: '🌙', genre: 'Dream Pop' },
        { id: 2, name: 'The Neon Hearts', followers: '1.8M', image: '⚡', genre: 'Synth Wave' },
        { id: 3, name: 'Beach Boys Revival', followers: '3.2M', image: '☀️', genre: 'Surf Rock' },
        { id: 4, name: 'Miles Ahead', followers: '1.5M', image: '🎷', genre: 'Jazz' },
        { id: 5, name: 'Thunder Strike', followers: '2.1M', image: '🎸', genre: 'Rock' },
        { id: 6, name: 'Symphony Orchestra', followers: '980K', image: '🎻', genre: 'Classical' },
        { id: 7, name: 'Urban Flow', followers: '2.7M', image: '🎤', genre: 'Hip Hop' },
        { id: 8, name: 'StarLight', followers: '4.3M', image: '⭐', genre: 'Pop' },
        { id: 9, name: 'Midnight Groove', followers: '1.2M', image: '🎹', genre: 'Electronic' },
        { id: 10, name: 'Acoustic Soul', followers: '890K', image: '🎵', genre: 'Acoustic' },
    ];

    return (
        <div className="max-w-screen-2xl mx-auto">
            <h2 className="text-4xl font-bold text-white mb-2">Your Artists</h2>
            <p className="text-white/60 mb-8">Following {artists.length} artists</p>
            
            <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-6">
                {artists.map((artist) => (
                    <div
                        key={artist.id}
                        className="bg-white/5 backdrop-blur-sm rounded-lg p-4 hover:bg-white/10 transition-all cursor-pointer group"
                    >
                        <div className="aspect-square bg-gradient-to-br from-purple-500/20 to-rose-500/20 rounded-full mb-4 flex items-center justify-center text-6xl">
                            {artist.image}
                        </div>
                        <h3 className="text-white font-bold text-sm mb-1 truncate text-center">
                            {artist.name}
                        </h3>
                        <p className="text-white/50 text-xs text-center">{artist.genre}</p>
                        <p className="text-white/40 text-xs mt-1 text-center">{artist.followers} followers</p>
                    </div>
                ))}
            </div>
        </div>
    );
};

export default Artists;
