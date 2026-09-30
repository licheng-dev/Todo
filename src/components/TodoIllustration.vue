<template>
  <svg
    class="illustration"
    viewBox="0 0 320 340"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    aria-hidden="true"
  >
    <defs>
      <linearGradient id="todoCard" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" stop-color="#ffffff" stop-opacity="0.96" />
        <stop offset="1" stop-color="#fbf8f4" stop-opacity="0.9" />
      </linearGradient>
      <filter id="todoSoft" x="-40%" y="-40%" width="180%" height="180%">
        <feDropShadow
          dx="0"
          dy="16"
          stdDeviation="18"
          flood-color="#c9beb2"
          flood-opacity="0.35"
        />
      </filter>
      <filter id="todoBlur" x="-60%" y="-60%" width="220%" height="220%">
        <feGaussianBlur stdDeviation="9" />
      </filter>
    </defs>

    <!-- soft ground shadow -->
    <ellipse
      cx="160"
      cy="312"
      rx="92"
      ry="13"
      fill="#d9cfc4"
      opacity="0.35"
      filter="url(#todoBlur)"
    />

    <!-- decorative arcs -->
    <path
      d="M46 156 A 122 122 0 0 1 152 56"
      stroke="#e8c49b"
      stroke-opacity="0.5"
      stroke-width="1.6"
      stroke-linecap="round"
    />
    <path
      d="M288 300 A 112 112 0 0 0 302 194"
      stroke="#e3d8cd"
      stroke-width="1.6"
      stroke-linecap="round"
    />

    <!-- stars -->
    <path
      d="M0 -7 C1 -2 2 -1 7 0 C2 1 1 2 0 7 C-1 2 -2 1 -7 0 C-2 -1 -1 -2 0 -7 Z"
      transform="translate(58 74)"
      fill="#e8c49b"
      fill-opacity="0.9"
    />
    <path
      d="M0 -7 C1 -2 2 -1 7 0 C2 1 1 2 0 7 C-1 2 -2 1 -7 0 C-2 -1 -1 -2 0 -7 Z"
      transform="translate(274 150) scale(0.72)"
      fill="#e8c49b"
      fill-opacity="0.8"
    />
    <path
      d="M0 -7 C1 -2 2 -1 7 0 C2 1 1 2 0 7 C-1 2 -2 1 -7 0 C-2 -1 -1 -2 0 -7 Z"
      transform="translate(92 302) scale(0.55)"
      fill="#e8c49b"
      fill-opacity="0.7"
    />

    <!-- floating card -->
    <g transform="rotate(-6 160 160)" filter="url(#todoSoft)">
      <rect
        x="55"
        y="40"
        width="210"
        height="240"
        rx="28"
        fill="url(#todoCard)"
        stroke="#ffffff"
        stroke-width="2"
      />
      <rect x="82" y="106" width="150" height="11" rx="5.5" fill="#ede9e4" />
      <rect x="82" y="144" width="112" height="11" rx="5.5" fill="#ede9e4" />
      <rect x="82" y="182" width="78" height="11" rx="5.5" fill="#ede9e4" />
    </g>

    <!-- gold badge with check -->
    <g filter="url(#todoSoft)">
      <circle cx="252" cy="258" r="30" fill="#e8c49b" />
      <path
        d="M240 258 l8 9 l16 -18"
        stroke="#ffffff"
        stroke-width="5"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </g>
  </svg>
</template>

<style scoped>
.illustration {
  width: 100%;
  height: auto;
  display: block;
}
</style>