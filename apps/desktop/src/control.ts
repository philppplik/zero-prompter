import { mount } from 'svelte';
import '@fontsource-variable/inter';
import '@fontsource-variable/jetbrains-mono';
import '@fontsource/atkinson-hyperlegible/400.css';
import '@fontsource/atkinson-hyperlegible/700.css';
import './app.css';
import App from './components/App.svelte';
import { applyUiTokens } from './lib/tokens';

applyUiTokens();

export default mount(App, { target: document.getElementById('app')! });
