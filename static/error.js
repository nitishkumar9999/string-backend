// Rate limit countdown timer
(function() {
    const countdown = document.getElementById('countdown');
    if (!countdown) return;

    let seconds = parseInt(countdown.dataset.seconds);
    
    const timer = setInterval(() => {
        seconds--;
        countdown.textContent = seconds + 's';
        
        if (seconds <= 0) {
            clearInterval(timer);
            countdown.textContent = 'Ready!';
            countdown.style.color = '#10b981';
        }
    }, 1000);
})();