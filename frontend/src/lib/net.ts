export let apiAddress = '';
import { dev } from '$app/environment';

// Set the API url to localhost for supporting
// dev environments
if (dev) {
	console.log('Development environment detected, switching to local server');
	apiAddress = 'http://localhost:8080';
}
