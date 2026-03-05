// Sidebar toggle for mobile
const toggle = document.getElementById('sidebar-toggle');
const sidebar = document.getElementById('sidebar');

if (toggle && sidebar) {
  toggle.addEventListener('click', () => {
    sidebar.classList.toggle('open');
  });

  // Close sidebar when clicking outside on mobile
  document.addEventListener('click', (e) => {
    if (sidebar.classList.contains('open') &&
        !sidebar.contains(e.target) &&
        !toggle.contains(e.target)) {
      sidebar.classList.remove('open');
    }
  });
}

// Highlight active nav link based on current URL
const currentPath = window.location.pathname;
document.querySelectorAll('.nav-list a').forEach(link => {
  const href = link.getAttribute('href');
  if (href && currentPath.endsWith(href.replace(/^\.\.?\/?/, '').replace(/^\//, ''))) {
    link.classList.add('active');
  }
});
