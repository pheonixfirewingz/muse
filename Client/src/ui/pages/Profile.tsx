import React from 'react';
import { UserCircleIcon, MusicalNoteIcon, HeartIcon, ClockIcon, ArrowRightOnRectangleIcon } from '@heroicons/react/24/outline';
import { useAuth } from '../contexts/AuthContext';

const Profile: React.FC = () => {
    const { username, logout } = useAuth();

    const stats = [
        { label: 'Total Tracks', value: '1,234', icon: MusicalNoteIcon },
        { label: 'Favorites', value: '89', icon: HeartIcon },
        { label: 'Listening Time', value: '247h', icon: ClockIcon },
    ];

    const recentActivity = [
        { id: 1, track: 'Starlight Symphony', artist: 'Aurora Moon', time: '2 hours ago' },
        { id: 2, track: 'Electric Dreams', artist: 'The Neon Hearts', time: '5 hours ago' },
        { id: 3, track: 'Thunder Road', artist: 'Thunder Strike', time: 'Yesterday' },
        { id: 4, track: 'Midnight Jazz', artist: 'Miles Ahead', time: 'Yesterday' },
        { id: 5, track: 'Urban Flow', artist: 'Urban Flow', time: '2 days ago' },
    ];

    return (
        <div className="max-w-screen-xl mx-auto">
            {/* Profile Header */}
            <div className="bg-gradient-to-br from-rose-500/20 to-purple-500/20 rounded-lg p-8 mb-8 backdrop-blur-sm">
                <div className="flex items-center gap-6">
                    <div className="w-32 h-32 bg-gradient-to-br from-rose-500 to-purple-600 rounded-full flex items-center justify-center">
                        <UserCircleIcon className="w-20 h-20 text-white" />
                    </div>
                    <div className="flex-1">
                        <h2 className="text-4xl font-bold text-white mb-2">{username || 'Music Lover'}</h2>
                        <p className="text-white/70 text-lg mb-4">Enjoying your music collection</p>
                        <div className="flex gap-3">
                            <button className="px-6 py-2 bg-rose-500 hover:bg-rose-600 text-white rounded-full font-medium transition-colors">
                                Edit Profile
                            </button>
                            <button 
                                onClick={logout}
                                className="px-6 py-2 bg-gray-700 hover:bg-gray-600 text-white rounded-full font-medium transition-colors flex items-center gap-2"
                            >
                                <ArrowRightOnRectangleIcon className="w-5 h-5" />
                                Logout
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            {/* Stats */}
            <div className="grid grid-cols-3 gap-4 mb-8">
                {stats.map((stat, index) => {
                    const Icon = stat.icon;
                    return (
                        <div
                            key={index}
                            className="bg-white/5 backdrop-blur-sm rounded-lg p-6 text-center hover:bg-white/10 transition-all"
                        >
                            <Icon className="w-8 h-8 text-rose-500 mx-auto mb-3" />
                            <div className="text-3xl font-bold text-white mb-1">{stat.value}</div>
                            <div className="text-white/50 text-sm">{stat.label}</div>
                        </div>
                    );
                })}
            </div>

            {/* Recent Activity */}
            <div className="bg-white/5 backdrop-blur-sm rounded-lg p-6">
                <h3 className="text-2xl font-bold text-white mb-6">Recent Activity</h3>
                <div className="space-y-4">
                    {recentActivity.map((activity) => (
                        <div
                            key={activity.id}
                            className="flex items-center justify-between p-4 rounded-lg hover:bg-white/5 transition-colors"
                        >
                            <div className="flex items-center gap-4">
                                <div className="w-12 h-12 bg-gradient-to-br from-rose-500/30 to-purple-500/30 rounded-md flex items-center justify-center">
                                    <MusicalNoteIcon className="w-6 h-6 text-white" />
                                </div>
                                <div>
                                    <div className="text-white font-medium">{activity.track}</div>
                                    <div className="text-white/50 text-sm">{activity.artist}</div>
                                </div>
                            </div>
                            <div className="text-white/40 text-sm">{activity.time}</div>
                        </div>
                    ))}
                </div>
            </div>

            {/* Settings Section */}
            <div className="bg-white/5 backdrop-blur-sm rounded-lg p-6 mt-6">
                <h3 className="text-2xl font-bold text-white mb-6">Preferences</h3>
                <div className="space-y-4">
                    <div className="flex items-center justify-between p-4 rounded-lg hover:bg-white/5">
                        <span className="text-white">Audio Quality</span>
                        <span className="text-rose-500 font-medium">High (320kbps)</span>
                    </div>
                    <div className="flex items-center justify-between p-4 rounded-lg hover:bg-white/5">
                        <span className="text-white">Notifications</span>
                        <span className="text-rose-500 font-medium">Enabled</span>
                    </div>
                    <div className="flex items-center justify-between p-4 rounded-lg hover:bg-white/5">
                        <span className="text-white">Download Quality</span>
                        <span className="text-rose-500 font-medium">Lossless</span>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default Profile;
