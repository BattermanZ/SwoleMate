# Stage 1: Build
FROM node:24-slim AS builder
WORKDIR /app

# Handle build-time variables
ARG VITE_API_URL
ENV VITE_API_URL=${VITE_API_URL}

# Install dependencies
COPY package*.json ./
RUN npm ci

# Copy source and build
COPY . .
RUN npm run build

# Stage 2: Serve with nginx
FROM nginx:alpine
# Copy the built static files
COPY --from=builder /app/build /usr/share/nginx/html
# Copy nginx configuration
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 2470

CMD ["nginx", "-g", "daemon off;"] 
