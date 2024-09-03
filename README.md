To set up the Paytm MERN project from the GitHub repository you mentioned, follow these detailed steps:

### 1. Clone the Repository

```bash
git clone https://github.com/sumana10/paytm-mern.git
cd paytm-mern
```

### 2. Set Up the Backend

1. **Navigate to the Backend Directory:**
   ```bash
   cd backend
   ```

2. **Create a `.env` File:**
   Add your MongoDB URL and Paytm credentials to a `.env` file:
   ```env
   MONGODB_URL=your_mongodb_url_here
   ```

3. **Install Dependencies:**
   ```bash
   npm install
   ```

4. **Start the Backend Server:**
   ```bash
   npm start
   ```

### 3. Set Up the Frontend

1. **Navigate to the Frontend Directory:**
   ```bash
   cd ../frontend
   ```

2. **Install Dependencies:**
   ```bash
   npm install
   ```

3. **Start the Frontend Development Server:**
   ```bash
   npm run dev
   ```

### 4. Test the Backend

1. **Import Postman Collection:**
   - Open Postman.
   - Click on "Import" and select the `Paytm.postman_collection.json` file from the `backend` directory (if it’s available) or ask for it if it’s not included.

2. **Run the Tests:**
   - Execute the requests in the Postman collection to test your backend.


