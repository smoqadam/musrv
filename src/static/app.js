const API_BASE = '/api';
const audio = document.getElementById('audio');
const volumeSlider = document.getElementById('volume-slider');
const playPauseBtn = document.getElementById('play-pause-btn');
const progressFill = document.getElementById('progress-fill');
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
let playlistFetchToken = 0;
let tracksLoading = false;
let tracksError = '';

if (volumeSlider) {
    volumeSlider.value = String(audio.volume || 1);
    volumeSlider.addEventListener('input', (event) => {
        audio.volume = parseFloat(event.target.value);
    });
}

setupMediaSessionHandlers();
setDocumentTitle(null);
loadFolder();

async function loadFolder(path = '') {
    try {
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

        updatePlaylistControls();
        await loadTracksFromM3U8(true);
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
    const disabledAttr = (enabled) => (enabled ? '' : 'disabled');

    playlistControlsEl.innerHTML = `
        <button class="playlist-btn" onclick="goBack()" ${disabledAttr(hasParent)}>back</button>
        <button class="playlist-btn" onclick="playPlaylist()" ${disabledAttr(!!playlistUrl)}>play all</button>
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
                return `<div class="track${isPlaying ? ' playing' : ''}" id="track-${track.playlistIndex}" onclick="playTrack(${track.playlistIndex})"><div class="track-name">${escapeHtml(track.name)}</div></div>`;
            })
            .join('');
    }

    if (!html) {
        html = '<div class="loading">no items found</div>';
    }

    playlistContentEl.innerHTML = html;
}

async function loadTracksFromM3U8(force = false) {
    if (!currentM3U8) {
        currentPlaylist = [];
        currentDisplayTracks = [];
        tracksLoading = false;
        tracksError = '';
        updatePlaylistContent();
        return currentPlaylist;
    }
    if (!force && currentPlaylist.length > 0) {
        return currentPlaylist;
    }

    const token = ++playlistFetchToken;
    try {
        tracksLoading = true;
        tracksError = '';
        updatePlaylistContent();

        const tracks = await parseM3U8(currentM3U8);
        if (token !== playlistFetchToken) {
            return currentPlaylist;
        }

        currentPlaylist = tracks;
        currentDisplayTracks = computeDisplayTracks(tracks);
        tracksLoading = false;
        tracksError = tracks.length === 0 ? 'No tracks found in playlist' : '';
        updatePlaylistContent();
        return currentPlaylist;
    } catch (error) {
        if (token !== playlistFetchToken) {
            return currentPlaylist;
        }
        tracksLoading = false;
        tracksError = 'error loading playlist';
        currentPlaylist = [];
        currentDisplayTracks = [];
        console.error('Error loading playlist:', error);
        updatePlaylistContent();
        return currentPlaylist;
    }
}

function playTrack(index) {
    if (!currentPlaylist.length) {
        return;
    }
    if (index < 0 || index >= currentPlaylist.length) {
        return;
    }

    currentTrackIndex = index;
    const track = currentPlaylist[index];
    audio.src = track.url;
    audio.play().catch((error) => {
        console.error('Playback error:', error);
    });
    updatePlayerInfo(track);
    updateTrackHighlight();
    updatePositionState();
}

async function playPlaylist() {
    if (!currentM3U8) {
        alert('Playlist URL not available');
        return;
    }
    const tracks = await loadTracksFromM3U8();
    if (!tracks.length) {
        alert('No tracks found in playlist');
        return;
    }
    playTrack(0);
}

function togglePlayback() {
    if (!audio.src) {
        if (currentTrackIndex >= 0) {
            playTrack(currentTrackIndex);
            return;
        }
        if (currentPlaylist.length) {
            playTrack(0);
        }
        return;
    }

    if (audio.paused) {
        audio.play().catch((error) => console.error('Playback error:', error));
    } else {
        audio.pause();
    }
}

function nextTrack() {
    if (!currentPlaylist.length) {
        return;
    }
    const hasNext = currentTrackIndex < currentPlaylist.length - 1;
    const nextIndex = hasNext ? currentTrackIndex + 1 : 0;
    playTrack(nextIndex);
}

function previousTrack() {
    if (!currentPlaylist.length) {
        return;
    }
    if (currentTrackIndex > 0) {
        playTrack(currentTrackIndex - 1);
    } else {
        playTrack(currentPlaylist.length - 1);
    }
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
        return;
    }
    titleEl.textContent = track.name;
    infoEl.textContent = `${currentTrackIndex + 1} of ${currentPlaylist.length} (playlist mode)`;
    setDocumentTitle(track);
    updateMediaSession(track);
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

audio.addEventListener('play', () => {
    if (playPauseBtn) {
        playPauseBtn.textContent = '⏸';
    }
    updatePlaybackState('playing');
    updatePositionState();
});

audio.addEventListener('pause', () => {
    if (playPauseBtn) {
        playPauseBtn.textContent = '▶';
    }
    updatePlaybackState('paused');
});

audio.addEventListener('ended', () => {
    updatePlaybackState('none');
    nextTrack();
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

async function parseM3U8(url) {
    try {
        const response = await fetch(url);
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        const text = await response.text();
        const lines = text
            .split('\n')
            .map((line) => line.trim())
            .filter(Boolean);
        const tracks = [];
        let currentTrackName = '';
        for (const line of lines) {
            if (line.startsWith('#EXTINF:')) {
                const match = line.match(/^#EXTINF:[^,]*,(.*)$/);
                currentTrackName = match ? match[1] : 'Unknown';
            } else if (!line.startsWith('#')) {
                const name = currentTrackName || 'Unknown';
                tracks.push({ name, url: line });
                currentTrackName = '';
            }
        }
        return tracks;
    } catch (error) {
        console.error('Error parsing M3U8:', error);
        return [];
    }
}

function relativePathFromUrl(url) {
    try {
        const parsed = new URL(url, window.location.href);
        const pathname = parsed.pathname || '';
        return pathname.startsWith('/') ? pathname.slice(1) : pathname;
    } catch (_error) {
        return url;
    }
}

function computeDisplayTracks(allTracks) {
    const normalizedPath = currentPath.replace(/\\+/g, '/');
    const encodedPrefix = normalizedPath
        ? `${normalizedPath.split('/').map((segment) => encodeURIComponent(segment)).join('/')}/`
        : '';
    return allTracks
        .map((track, index) => {
            const relPath = relativePathFromUrl(track.url);
            if (encodedPrefix && !relPath.startsWith(encodedPrefix)) {
                return null;
            }
            const tail = encodedPrefix ? relPath.slice(encodedPrefix.length) : relPath;
            if (!tail || tail.includes('/')) {
                return null;
            }
            return {
                name: track.name,
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
    if (track && track.name) {
        document.title = `${track.name} · musrv`;
    } else {
        document.title = baseDocumentTitle;
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
            const folder = currentFolderLabel();
            navigator.mediaSession.metadata = new MediaMetadata({
                title: track.name,
                artist: folder,
                album: folder,
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
