<template>
  <div class="loading">
    <div class="loading-header">
      <div class="spinner"></div>
      <p>Processing repository...</p>
    </div>
  </div>
</template>

<style scoped>
.loading {
  padding: 24px;
  text-align: center;
}

.loading-header {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
}

.loading-header p {
  margin: 0;
}

.spinner {
  flex-shrink: 0;
  width: 20px;
  height: 20px;
  border: 2px solid var(--vp-c-brand-1);
  border-radius: 50%;
  border-top-color: transparent;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
