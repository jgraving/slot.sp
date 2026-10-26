// Only play documentation clips when requested. Pause them when leaving the tab.
document.addEventListener('visibilitychange', () => {
  if (document.hidden) document.querySelectorAll('video').forEach(video => video.pause());
});
