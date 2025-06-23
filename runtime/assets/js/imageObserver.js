(function() {
    'use strict';

    const processedKey = 'data-image-handler-attached';

    async function fetchImageAsBlob(url) {
        try {
            const response = await fetch(url);
            if (!response.ok) {
                return null;
            }
            const blob = await response.blob();
            return URL.createObjectURL(blob);
        } catch (error) {
            return null;
        }
    }

    function setupImageHandlers(imgElement) {
        if (imgElement.hasAttribute(processedKey)) {
            return;
        }
        
        imgElement.setAttribute(processedKey, 'true');

        const wrapper = imgElement.closest('.cover-wrapper');
        if (!wrapper) {
            return;
        }

        const fallbackIcon = wrapper.querySelector('.fallback-icon');

        // Check if image is initially hidden
        const initialDisplay = window.getComputedStyle(imgElement).display;
        const isInitiallyHidden = initialDisplay === 'none';

        const onImageLoad = () => {
            if (fallbackIcon) {
                fallbackIcon.style.display = 'none';
            }
            imgElement.style.display = 'block';
            console.log(`Image loaded successfully: ${imgElement.src}`);
            cleanup();
        };

        const onImageError = () => {
            imgElement.style.display = 'none';
            if (fallbackIcon) {
                fallbackIcon.style.display = 'inline-block';
            }
            console.log(`WARNING: this is intentional behavior when the server can not find an image based on require Muse has try to stop the browser from erroring but have not had luck`);
            console.log(`Image failed to load: ${imgElement.src}`);
            cleanup();
        };

        const cleanup = () => {
            imgElement.removeEventListener('load', onImageLoad);
            imgElement.removeEventListener('error', onImageError);
        };

        imgElement.addEventListener('load', onImageLoad);
        imgElement.addEventListener('error', onImageError);

        // If the image is already cached/complete, the load event might not fire
        if (imgElement.complete && imgElement.naturalWidth > 0) {
            onImageLoad();
        } else if (imgElement.complete && imgElement.naturalWidth === 0 && imgElement.src) {
            onImageError();
        } else {
            // If image is initially hidden, show fallback icon
            if (isInitiallyHidden && fallbackIcon) {
                fallbackIcon.style.display = 'inline-block';
            }
        }
    }

    async function createImageFromDataSrc(wrapper) {
        const dataImgSrc = wrapper.dataset.imgSrc;
        if (!dataImgSrc) {
            return;
        }
        
        const img = new Image();
        img.alt = "Cover";
        img.style.display = 'none'; // Start hidden
        
        wrapper.appendChild(img);
        
        // Fetch the image data and create blob URL
        const blobUrl = await fetchImageAsBlob(dataImgSrc);
        if (blobUrl) {
            img.src = blobUrl;
            setupImageHandlers(img);
        } else {
            // If fetch failed, show fallback icon
            const fallbackIcon = wrapper.querySelector('.fallback-icon');
            if (fallbackIcon) {
                fallbackIcon.style.display = 'inline-block';
            }
            console.log(`WARNING: this is intentional behavior when the server can not find an image based on require Muse has try to stop the browser from erroring but have not had luck`);
            console.log(`Image failed to load: ${dataImgSrc}`);
        }
    }

    function handleCoverWrapper(wrapper) {
        const img = wrapper.querySelector('img');
        if (img) {
            setupImageHandlers(img);
        } else {
            // Check if wrapper has data-img-src attribute
            if (wrapper.dataset.imgSrc) {
                createImageFromDataSrc(wrapper);
            } else {
                // If image is not there and no data-img-src, observe the wrapper for it to be added
                const wrapperObserver = new MutationObserver((mutationsList) => {
                    for (const mutation of mutationsList) {
                        for (const node of mutation.addedNodes) {
                            if (node.tagName === 'IMG') {
                                setupImageHandlers(node);
                                // Once we find the img, we can stop observing this specific wrapper.
                                wrapperObserver.disconnect(); 
                            }
                        }
                    }
                });
                wrapperObserver.observe(wrapper, { childList: true });
            }
        }
    }

    const mainObserver = new MutationObserver((mutationsList, observer) => {
        for (const mutation of mutationsList) {
            if (mutation.type === 'childList') {
                for (const node of mutation.addedNodes) {
                    if (node.nodeType === Node.ELEMENT_NODE) {
                        if (node.matches('.cover-wrapper')) {
                            handleCoverWrapper(node);
                        }
                        
                        const coverWrappers = node.querySelectorAll('.cover-wrapper');
                        if (coverWrappers.length > 0) {
                            coverWrappers.forEach(handleCoverWrapper);
                        }
                    }
                }
            }
        }
    });

    mainObserver.observe(document.body, {
        childList: true,
        subtree: true
    });

    // Handle existing elements on load
    const existingWrappers = document.querySelectorAll('.cover-wrapper');
    existingWrappers.forEach((wrapper, index) => {
        handleCoverWrapper(wrapper);
    });

})(); 