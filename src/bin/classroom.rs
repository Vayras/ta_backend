use std::{env};
use serde::Deserialize;
use dotenv::dotenv;
use octocrab::Octocrab;
use thiserror::Error;

// Represents a GitHub organization, often linked to a classroom
#[derive(Debug, Deserialize)]
struct ClassroomOrganization {
    id: u64,
    login: String,
    html_url: String,
    avatar_url: String,
    node_id: String,
}

// Represents a GitHub Classroom
#[derive(Debug, Deserialize)]
struct Classroom {
    id: u64,
    name: String,
    archived: bool,
    organization: ClassroomOrganization, // Nested organization details
    url: String, // URL to the classroom on classroom.github.com
}

// --- Custom Error type for better error handling ---
#[derive(Debug, Error)]
enum ClassroomError {
    #[error("GitHub API error: {0}")]
    Octocrab(#[from] octocrab::Error),
    #[error("Environment variable GITHUB_PAT not set")]
    MissingToken,
    #[error("Failed to parse API response: {0}")]
    ParseError(#[from] serde_json::Error),
    #[error("Request failed: {0}")]
    RequestFailed(String),
}


pub fn get_env() -> String{
    dotenv().ok();

    let api_key = env::var("GITHUB_TOKEN");

    let result = match api_key {
        Ok(val) => val,
        Err(e) => "error".to_string() , 
    };

    
    return result
}

#[tokio::main]
async fn main() -> Result<(), ClassroomError>{
    let key:String = get_env();

    let octocrab = Octocrab::builder().personal_token(key).build()?;

    let classrooms_response: Result<Vec<Classroom>, octocrab::Error> = octocrab.get("/classrooms", None::<&()>).await;
    
     match classrooms_response {
        Ok(classrooms) => {
            if classrooms.is_empty() {
                println!("No classrooms found or you may not have access to any.");
            } else {
                println!("\nFound the following classrooms:");
                for classroom in classrooms {
                    println!("------------------------------------");
                    println!("Classroom ID: {}", classroom.id);
                    println!("Name:         {}", classroom.name);
                    println!("Archived:     {}", classroom.archived);
                    println!("Organization: {} ({})", classroom.organization.login, classroom.organization.html_url);
                    println!("Classroom URL:{}", classroom.url);
                }
                println!("------------------------------------");
            }
        }
        Err(e) => {
            eprintln!("Error fetching classrooms: {}", e);

            if let octocrab::Error::GitHub { source, .. } = &e {
                eprintln!("GitHub API Error Details: {:?}", source.message);
            }
            return Err(ClassroomError::Octocrab(e));
        }
    }
    
    Ok(())
}