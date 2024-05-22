export let apiAddress = '';
import { dev } from '$app/environment';

// Set the API url to localhost for supporting
// dev environments
if (dev) {
	console.log('Development environment detected, switching to local server');
	apiAddress = 'http://127.0.0.1:8080';
}
