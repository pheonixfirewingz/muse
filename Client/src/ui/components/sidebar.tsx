import React from 'react';
import { HomeIcon, QueueListIcon, HeartIcon, MusicalNoteIcon, UserCircleIcon } from '@heroicons/react/24/outline';

interface SideBarProps {
    currentPage: string;
    onNavigate: (page: string) => void;
}

export function SideBar({ currentPage, onNavigate }: SideBarProps) {
    const menuItems = [
        { key: 'library', label: 'Library', icon: HomeIcon },
        { key: 'playlists', label: 'Playlists', icon: QueueListIcon },
        { key: 'favorites', label: 'Favorites', icon: HeartIcon },
        { key: 'artists', label: 'Artists', icon: MusicalNoteIcon },
    ];

    return (
        <div className="w-64 bg-black/40 backdrop-blur-md border-r border-white/10 h-full flex flex-col">
            <div className="p-6 border-b border-white/10">
                <h1 className="text-2xl font-bold text-white">Muse</h1>
                <p className="text-xs text-white/50 mt-1">Music Streaming</p>
            </div>
            <nav className="flex-1 px-3 py-4">
                {menuItems.map((item) => {
                    const Icon = item.icon;
                    const isSelected = currentPage === item.key;
                    return (
                        <button
                            key={item.key}
                            onClick={() => onNavigate(item.key)}
                            className={`w-full flex items-center gap-3 px-4 py-3 rounded-lg mb-1 transition-all ${
                                isSelected
                                    ? 'bg-rose-500/20 text-rose-500'
                                    : 'text-white/70 hover:text-white hover:bg-white/5'
                            }`}
                        >
                            <Icon className="w-5 h-5" />
                            <span className="text-sm font-medium">{item.label}</span>
                        </button>
                    );
                })}
            </nav>
            <div className="px-3 pb-4 border-t border-white/10 pt-4">
                <button
                    onClick={() => onNavigate('profile')}
                    className={`w-full flex items-center gap-3 px-4 py-3 rounded-lg transition-all ${
                        currentPage === 'profile'
                            ? 'bg-rose-500/20 text-rose-500'
                            : 'text-white/70 hover:text-white hover:bg-white/5'
                    }`}
                >
                    <UserCircleIcon className="w-5 h-5" />
                    <span className="text-sm font-medium">Profile</span>
                </button>
            </div>
        </div>
    );
}

export default SideBar;
