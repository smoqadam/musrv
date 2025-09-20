const API_BASE = '/api';
const audio = document.getElementById('audio');
const playPauseBtn = document.getElementById('play-pause-btn');
const progressFill = document.getElementById('progress-fill');
const PLAY_ICON = '<svg width="28" height="28" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" aria-hidden="true"><path d="M8 5v14l11-7-11-7z" fill="currentColor"/></svg>';
const PAUSE_ICON = '<svg width="28" height="28" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" aria-hidden="true"><path d="M7 5h4v14H7zM13 5h4v14h-4z" fill="currentColor"/></svg>';
const shuffleBtn = document.getElementById('shuffle-btn');
const artworkContainer = document.getElementById('now-playing-artwork');
const artworkImg = document.getElementById('now-playing-artwork-img');
const breadcrumbEl = document.getElementById('breadcrumb');
const playlistContentEl = document.getElementById('playlist-content');
const playlistControlsEl = document.getElementById('playlist-controls');
const baseDocumentTitle = document.title;
const mediaSessionSupported = 'mediaSession' in navigator;
const positionStateSupported =
    mediaSessionSupported && typeof navigator.mediaSession.setPositionState === 'function';

let currentPath = '';
let currentAlbums = [];
let currentPlaylist = [];
let currentDisplayTracks = [];
let currentTrackIndex = -1;
let currentM3U8 = '';
let tracksLoading = false;
let tracksError = '';
let playQueue = [];
let queueIndex = -1;
let isShuffleEnabled = false;

function cloneTrack(track) {
    if (!track) {
        return null;
    }
    return {
        name: track.name,
        displayName: track.displayName || track.name,
        relative_path: track.relative_path || track.name || '',
        url: track.url,
        title: track.title || null,
        artist: track.artist || null,
        album: track.album || null,
        duration: typeof track.duration === 'number' ? track.duration : null,
        artwork_url: track.artwork_url || null,
    };
}

function clearQueue() {
    playQueue = [];
    queueIndex = -1;
    isShuffleEnabled = false;
    audio.pause();
    audio.removeAttribute('src');
    audio.load();
    setPlayPauseVisual(false);
    currentTrackIndex = -1;
    updatePlayerInfo(null);
    updateTrackHighlight();
    updateShuffleButton();
}

function setQueueFromTracks(tracks, startIndex = 0) {
    const cloned = tracks.map(cloneTrack).filter(Boolean);
    if (!cloned.length) {
        clearQueue();
        return;
    }
    playQueue = cloned;
    queueIndex = Math.max(0, Math.min(startIndex, playQueue.length - 1));
    playQueueTrack(queueIndex);
    updateShuffleButton();
}

function playQueueTrack(index, autoplay = true) {
    if (!playQueue.length) {
        clearQueue();
        return;
    }
    queueIndex = ((index % playQueue.length) + playQueue.length) % playQueue.length;
    const track = playQueue[queueIndex];
    if (!track) {
        return;
    }
    if (audio.src !== track.url) {
        audio.src = track.url;
    }
    if (autoplay) {
        audio.play().catch((error) => {
            console.error('Playback error:', error);
        });
    } else {
        audio.pause();
    }
    const playlistIdx = currentPlaylist.findIndex((item) => item.url === track.url);
    currentTrackIndex = playlistIdx;
    updatePlayerInfo(track);
    updateTrackHighlight();
    updatePositionState();
}

function updateShuffleButton() {
    if (!shuffleBtn) {
        return;
    }
    const disabled = playQueue.length <= 1;
    if (disabled && isShuffleEnabled) {
        isShuffleEnabled = false;
    }
    shuffleBtn.disabled = disabled;
    shuffleBtn.classList.toggle('active', isShuffleEnabled && !disabled);
    shuffleBtn.setAttribute('aria-label', isShuffleEnabled ? 'Shuffle on' : 'Shuffle off');
    shuffleBtn.title = disabled
        ? 'Need at least two tracks to shuffle'
        : isShuffleEnabled
            ? 'Shuffle enabled'
            : 'Shuffle disabled';
}

function addTrackToQueue(event, playlistIndex) {
    if (event) {
        event.preventDefault();
        event.stopPropagation();
    }
    if (!currentPlaylist.length) {
        return;
    }
    const track = currentPlaylist[playlistIndex];
    if (!track) {
        return;
    }
    const cloned = cloneTrack(track);
    const wasEmpty = playQueue.length === 0;
    playQueue.push(cloned);
    if (wasEmpty) {
        queueIndex = 0;
        playQueueTrack(queueIndex);
    }
    updateShuffleButton();
}

setupMediaSessionHandlers();
setDocumentTitle(null);
setPlayPauseVisual(false);
updatePlayerInfo(null);
updateShuffleButton();
loadFolder();

async function loadFolder(path = '') {
    try {
        tracksLoading = true;
        tracksError = '';
        playlistContentEl.innerHTML = '<div class="loading">loading music library...</div>';
        const url = path ? `${API_BASE}/folder?path=${encodeURIComponent(path)}` : `${API_BASE}/folder`;
        const response = await fetch(url);
        if (!response.ok) {
            throw new Error('Failed to fetch folder');
        }
        const data = await response.json();
        currentPath = data.path;
        currentAlbums = data.albums || [];
        currentM3U8 = data.m3u8 || '';
        breadcrumbEl.textContent = currentPath || 'home';

        if (!audio.src) {
            setDocumentTitle(null);

            updateMediaSession(null);
        }

        const tracks = Array.isArray(data.tracks) ? data.tracks : [];
        currentPlaylist = tracks.map((track, index) => ({
            ...track,
            displayName: track.display_name || track.name,
            playlistIndex: index,
        }));
        currentDisplayTracks = computeDisplayTracks(currentPlaylist);
        tracksLoading = false;
        tracksError = !currentPlaylist.length && !currentAlbums.length ? 'no tracks found' : '';

        updatePlaylistControls();
        updatePlaylistContent();
        if (playQueue.length && queueIndex >= 0 && playQueue[queueIndex]) {
            updatePlayerInfo(playQueue[queueIndex]);
        } else {
            updatePlayerInfo(null);
        }
        updateTrackHighlight();
        updateShuffleButton();
        scrollPlaylistToTop();
    } catch (error) {
        console.error('Error loading folder:', error);
        tracksLoading = false;
        tracksError = 'failed to load folder';
        currentAlbums = [];
        currentPlaylist = [];
        currentDisplayTracks = [];
        updatePlaylistControls();
        updatePlaylistContent();
    }
}

function updatePlaylistControls() {
    const hasParent = Boolean(currentPath);
    const playlistUrl = currentPlaylistUrl();
    const hasTracks = currentPlaylist.length > 0;
    const disabledAttr = (enabled) => (enabled ? '' : 'disabled');

    playlistControlsEl.innerHTML = `
        <button class="playlist-btn" onclick="goBack()" ${disabledAttr(hasParent)}>back</button>
        <button class="playlist-btn" onclick="playPlaylist()" ${disabledAttr(hasTracks)}>play all</button>
        <button class="playlist-btn" id="copy-btn" onclick="copyM3U8()" ${disabledAttr(!!playlistUrl)}>📋 copy m3u8</button>
        <button class="playlist-btn" id="download-btn" onclick="downloadM3U8()" ${disabledAttr(!!playlistUrl)}>⬇ download m3u8</button>
    `;
}

function updatePlaylistContent() {
    let html = '';

    if (currentAlbums.length > 0) {
        html += currentAlbums
            .map((album) => {
                const escapedPath = escapeJsString(album.path);
                return `<div class="folder" onclick="loadFolder('${escapedPath}')"><div class="folder-name">📁 ${escapeHtml(album.name)}</div></div>`;
            })
            .join('');
    }

    if (tracksLoading) {
        html += '<div class="loading">loading tracks...</div>';
    } else if (tracksError) {
        html += `<div class="error">${escapeHtml(tracksError)}</div>`;
    } else if (currentDisplayTracks.length > 0) {
        html += currentDisplayTracks
            .map((track) => {
                const isPlaying = track.playlistIndex === currentTrackIndex;
                return `
                    <div class="track${isPlaying ? ' playing' : ''}" id="track-${track.playlistIndex}" onclick="playTrack(${track.playlistIndex})">
                        <div class="track-name">${escapeHtml(track.name)}</div>
                    </div>
                `;
            })
            .join('');
    }

    if (!html) {
        html = '<div class="loading">no items found</div>';
    }

    playlistContentEl.innerHTML = html;
}

function playTrack(index) {
    if (!currentPlaylist.length) {
        return;
    }
    if (index < 0 || index >= currentPlaylist.length) {
        return;
    }
    const upcomingTracks = currentPlaylist.slice(index);
    setQueueFromTracks(upcomingTracks, 0);
}

function playPlaylist() {
    if (!currentPlaylist.length) {
        if (!currentM3U8) {
            alert('Playlist URL not available');
        } else {
            alert('No tracks found in playlist');
        }
        return;
    }
    setQueueFromTracks(currentPlaylist, 0);
}

function togglePlayback() {
    if (!audio.src || !audio.src.length) {
        if (playQueue.length) {
            const start = queueIndex >= 0 ? queueIndex : 0;
            playQueueTrack(start);
        } else if (currentPlaylist.length) {
            setQueueFromTracks(currentPlaylist, 0);
        }
        return;
    }

    if (audio.paused) {
        audio.play().catch((error) => console.error('Playback error:', error));
    } else {
        audio.pause();
    }
}

function nextTrack(forceAutoPlay = false) {
    if (!playQueue.length) {
        return;
    }
    const shouldPlay = forceAutoPlay || !audio.paused;
    let nextIndex;
    if (isShuffleEnabled && playQueue.length > 1) {
        do {
            nextIndex = Math.floor(Math.random() * playQueue.length);
        } while (nextIndex === queueIndex);
    } else {
        nextIndex = queueIndex + 1;
    }
    playQueueTrack(nextIndex, shouldPlay);
}

function previousTrack(forceAutoPlay = false) {
    if (!playQueue.length) {
        return;
    }
    const shouldPlay = forceAutoPlay || !audio.paused;
    const prevIndex = queueIndex - 1;
    playQueueTrack(prevIndex < 0 ? playQueue.length - 1 : prevIndex, shouldPlay);
}

function goBack() {
    if (!currentPath) {
        return;
    }
    const idx = currentPath.lastIndexOf('/');
    const parent = idx >= 0 ? currentPath.slice(0, idx) : '';
    loadFolder(parent);
}

function seekTo(event) {
    if (!audio.duration) {
        return;
    }
    const rect = event.currentTarget.getBoundingClientRect();
    const percentage = (event.clientX - rect.left) / rect.width;
    audio.currentTime = percentage * audio.duration;
    updateProgress();
    updatePositionState();
}

function updatePlayerInfo(track) {
    const titleEl = document.getElementById('track-title');
    const infoEl = document.getElementById('track-info');
    if (!track) {
        titleEl.textContent = 'no track selected';
        infoEl.textContent = 'ready to play';
        setDocumentTitle(null);
        updateMediaSession(null);
        setArtwork(null);
        return;
    }
    const displayName = track.displayName || track.name;
    titleEl.textContent = displayName;
    const metaParts = [];
    if (track.artist) {
        metaParts.push(track.artist);
    }
    if (track.album) {
        metaParts.push(track.album);
    }
    if (!metaParts.length) {
        if (queueIndex >= 0 && playQueue.length > 1) {
            metaParts.push(`${queueIndex + 1} of ${playQueue.length}`);
        } else {
            metaParts.push('playlist mode');
        }
    }
    infoEl.textContent = metaParts.join(' · ');
    setDocumentTitle(track);
    updateMediaSession(track);
    setArtwork(track.artwork_url || null);
}

function updateTrackHighlight() {
    updatePlaylistContent();
}

function updateProgress() {
    if (!progressFill) {
        return;
    }
    if (!audio.duration || Number.isNaN(audio.duration)) {
        progressFill.style.width = '0%';
        return;
    }
    const percentage = (audio.currentTime / audio.duration) * 100;
    progressFill.style.width = `${percentage}%`;
}

function setPlayPauseVisual(isPlaying) {
    if (!playPauseBtn) {
        return;
    }
    playPauseBtn.innerHTML = isPlaying ? PAUSE_ICON : PLAY_ICON;
    playPauseBtn.setAttribute('aria-label', isPlaying ? 'Pause' : 'Play');
}

audio.addEventListener('play', () => {
    setPlayPauseVisual(true);
    updatePlaybackState('playing');
    updatePositionState();
});

audio.addEventListener('pause', () => {
    setPlayPauseVisual(false);
    updatePlaybackState('paused');
});

audio.addEventListener('ended', () => {
    updatePlaybackState('none');
    nextTrack(true);
});

audio.addEventListener('timeupdate', () => {
    updateProgress();
    updatePositionState();
});

audio.addEventListener('loadedmetadata', updatePositionState);

audio.addEventListener('durationchange', updatePositionState);

audio.addEventListener('ratechange', updatePositionState);

async function rescanLibrary() {
    const button = document.getElementById('rescan-btn');
    if (button) {
        button.disabled = true;
        button.textContent = 'rescanning...';
    }
    try {
        const response = await fetch(`${API_BASE.replace('/api', '')}/admin/rescan`);
        if (response.ok) {
            await loadFolder(currentPath);
        } else {
            throw new Error('rescan failed');
        }
    } catch (error) {
        alert('error rescanning library: ' + error.message);
    } finally {
        if (button) {
            button.disabled = false;
            button.textContent = 'rescan';
        }
    }
}

function currentPlaylistUrl() {
    return currentM3U8 || '';
}

function copyM3U8() {
    const playlistUrl = currentPlaylistUrl();
    if (!playlistUrl) {
        return;
    }
    const btn = document.getElementById('copy-btn');
    const reset = () => {
        if (btn) {
            btn.classList.remove('copied');
            btn.textContent = '📋 copy m3u8';
        }
    };
    const markCopied = () => {
        if (btn) {
            btn.classList.add('copied');
            btn.textContent = '✅ copied';
            setTimeout(reset, 2000);
        }
    };
    if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(playlistUrl).then(markCopied).catch((error) => {
            console.error('Clipboard error:', error);
        });
    } else {
        const ta = document.createElement('textarea');
        ta.value = playlistUrl;
        document.body.appendChild(ta);
        ta.select();
        document.execCommand('copy');
        document.body.removeChild(ta);
        markCopied();
    }
}

async function downloadM3U8() {
    const playlistUrl = currentPlaylistUrl();
    if (!playlistUrl) {
        return;
    }
    try {
        const response = await fetch(playlistUrl);
        if (!response.ok) {
            throw new Error('failed to fetch playlist');
        }
        const text = await response.text();
        const blob = new Blob([text], { type: 'audio/x-mpegurl;charset=utf-8' });
        const objectUrl = URL.createObjectURL(blob);
        const anchor = document.createElement('a');
        anchor.href = objectUrl;
        anchor.download = `${sanitizeFileName(currentFolderLabel() || 'playlist')}.m3u8`;
        document.body.appendChild(anchor);
        anchor.click();
        document.body.removeChild(anchor);
        URL.revokeObjectURL(objectUrl);
    } catch (error) {
        console.error('Download failed:', error);
        window.open(playlistUrl, '_blank');
    }
}

function computeDisplayTracks(allTracks) {
    const normalizedPath = currentPath.replace(/\\+/g, '/');
    const prefix = normalizedPath ? `${normalizedPath}/` : '';
    return allTracks
        .map((track, index) => {
            const relPath = (track.relative_path || '').replace(/\\+/g, '/');
            if (prefix) {
                if (!relPath.startsWith(prefix)) {
                    return null;
                }
                const tail = relPath.slice(prefix.length);
                if (!tail || tail.includes('/')) {
                    return null;
                }
            } else if (relPath.includes('/')) {
                return null;
            }
            return {
                name: track.displayName || track.name,
                url: track.url,
                playlistIndex: index,
            };
        })
        .filter(Boolean);
}

function scrollPlaylistToTop() {
    if (playlistContentEl) {
        playlistContentEl.scrollTo({ top: 0, behavior: 'smooth' });
    }
}

function currentFolderLabel() {
    if (!currentPath) {
        return 'library';
    }
    const parts = currentPath.split('/').filter(Boolean);
    return parts[parts.length - 1] || 'library';
}

function sanitizeFileName(name) {
    return name.replace(/[\\/:*?"<>|]/g, '-');
}

function escapeHtml(str) {
    return String(str)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#39;');
}

function escapeJsString(value) {
    return String(value)
        .replace(/\\/g, '\\\\')
        .replace(/'/g, "\\'");
}

function setDocumentTitle(track) {
    if (track && (track.displayName || track.name)) {
        const displayName = track.displayName || track.name;
        document.title = `${displayName} · musrv`;
    } else {
        document.title = baseDocumentTitle;
    }
}

function setArtwork(url) {
    if (!artworkContainer || !artworkImg) {
        return;
    }
    if (url) {
        artworkImg.src = url;
        artworkContainer.classList.add('has-art');
    } else {
        artworkImg.removeAttribute('src');
        artworkImg.src = '';
        artworkContainer.classList.remove('has-art');
    }
}

function setupMediaSessionHandlers() {
    if (!mediaSessionSupported) {
        return;
    }
    const activeHandlers = {
        play: () => {
            audio.play().catch(() => {});
        },
        pause: () => {
            audio.pause();
        },
        previoustrack: () => {
            previousTrack();
        },
        nexttrack: () => {
            nextTrack();
        },
    };

    Object.entries(activeHandlers).forEach(([action, handler]) => {
        trySetActionHandler(action, handler);
    });

    ['stop', 'seekto', 'seekbackward', 'seekforward'].forEach((action) => {
        trySetActionHandler(action, null);
    });
}

function trySetActionHandler(action, handler) {
    try {
        navigator.mediaSession.setActionHandler(action, handler);
    } catch (error) {
        if (handler !== null) {
            console.debug(`Media Session action '${action}' unsupported`, error);
        }
    }
}

function updateMediaSession(track) {
    if (!mediaSessionSupported) {
        return;
    }
    try {
        if (track) {
            const displayName = track.displayName || track.name;
            const artist = track.artist || currentFolderLabel();
            const album = track.album || currentFolderLabel();
            const artwork = track.artwork_url
                ? [
                      {
                          src: track.artwork_url,
                      },
                  ]
                : [];
            navigator.mediaSession.metadata = new MediaMetadata({
                title: displayName,
                artist,
                album,
                artwork,
            });
        } else {
            navigator.mediaSession.metadata = null;
        }
    } catch (error) {
        console.debug('Unable to update media metadata:', error);
    }
}

function updatePlaybackState(state) {
    if (!mediaSessionSupported) {
        return;
    }
    try {
        navigator.mediaSession.playbackState = state;
    } catch (error) {
        console.debug('Unable to update playback state:', error);
    }
}

function updatePositionState() {
    if (!positionStateSupported) {
        return;
    }
    if (!audio.duration || !Number.isFinite(audio.duration)) {
        return;
    }
    try {
        navigator.mediaSession.setPositionState({
            duration: audio.duration,
            playbackRate: audio.playbackRate,
            position: audio.currentTime,
        });
    } catch (error) {
        console.debug('Unable to update position state:', error);
    }
}

function toggleShuffle() {
    if (playQueue.length <= 1) {
        isShuffleEnabled = false;
        updateShuffleButton();
        return;
    }
    isShuffleEnabled = !isShuffleEnabled;
    updateShuffleButton();
}

// Expose functions on window for inline handlers
window.loadFolder = loadFolder;
window.playTrack = playTrack;
window.playPlaylist = playPlaylist;
window.togglePlayback = togglePlayback;
window.nextTrack = nextTrack;
window.previousTrack = previousTrack;
window.goBack = goBack;
window.seekTo = seekTo;
window.rescanLibrary = rescanLibrary;
window.copyM3U8 = copyM3U8;
window.downloadM3U8 = downloadM3U8;
window.addTrackToQueue = addTrackToQueue;
window.toggleShuffle = toggleShuffle;
