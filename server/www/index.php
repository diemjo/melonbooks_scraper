<!DOCTYPE html>
<html>
<meta charset="UTF-8">
<head>
    <link rel="stylesheet" type="text/css" href="melonbooks.css">
    <link rel="stylesheet" type="text/css" href="/node_modules/material-components-web/dist/material-components-web.css">
    <link href="https://fonts.googleapis.com/icon?family=Material+Icons" rel="stylesheet" />
    <script src="/node_modules/material-components-web/dist/material-components-web.js"></script>
    <link rel="icon" type="image/png" href="/images/favicon.png">
    <script src="melonbooks.js"></script>

    <title display=none>Melonbooks</title>
</head>
<body>
    <?php
    $useragent = $_SERVER['HTTP_USER_AGENT'];
    $iPhone = stripos($useragent, "iPhone");
    $Android = stripos($useragent, "Android");
    $is_phone = ($iPhone||$Android);

    $dbpath = __DIR__.DIRECTORY_SEPARATOR."db".DIRECTORY_SEPARATOR."melonbooks.db";
    $pdo = new PDO("sqlite:$dbpath");
    $pdo->setAttribute( PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION );
    $artist_query = $pdo->query("SELECT name from artists");

    /*if ($dbcon->connect_error) {
        echo("Database connection failed " . $dbcon->connect_error);
    }*/

    function update_artist($a) {
        //Todo
    }

    $artists = array();
    function reset_artist_names() {
        GLOBAL $pdo, $artists;
        $artist_query = $pdo->query("SELECT name FROM artists");
        $artist_query->execute();
        $artists = array();
        foreach($artist_query as $entry) {
            $artist = $entry['name'];
            $artists[] = $artist;
        }
    }

    reset_artist_names();

    switch($_POST["submit"]) {
        case 'Add Artist': {
            if ($_POST["add_artist"] && $_POST["site"]) {
                $artist = trim($_POST["add_artist"]);
                $site = trim($_POST["site"]);
                //echo "trying to insert " . $artist . " for " . $site;
                $insert_artist_stmt = $pdo->prepare("INSERT INTO artists (name, site) VALUES (?, ?)");
                if ($insert_artist_stmt->execute([$artist, $site])) {
                    update_artist($artist);
                    //echo "success";
                }
                reset_artist_names();
            }
            break;
        }
        case 'Remove Artist': {
            if ($_POST["artist_names"]) {
                $artist = trim($_POST["artist_names"]);
                $remove_artist_stmt = $pdo->prepare("DELETE FROM artists WHERE name = (?) AND site = (?)");
                if ($remove_artist_stmt->execute([$artist, "melonbooks"])) {
                }
                reset_artist_names();
            }
        }
        default: {}
    }

    ?>

    <form action='/melonbooks/index.php' method='post' id='add_artist'>
        <input autocorrect='off' autocomplete='off' name='add_artist' spellcheck='false' maxlength='512'>
        <select name='site' id='site' form='add_artist'>
            <option value='melonbooks'>Melonbooks</option>
        </select>
        <input type='submit' name="submit" value='Add Artist'>
    </form>
    <br>
    <div>
    <?php
    echo "<label for='artist_names'>Choose an artist </label>";
    echo "<select name='artist_names' id='artist_names' form='remove_artist' onchange='this.options[this.selectedIndex].value && (window.location = \"/melonbooks/index.php?artist=\" + this.options[this.selectedIndex].value) || (window.location = \"/melonbooks/index.php\");'>";
    echo "<option value=''>All</option>";
    foreach ($artists as $artist) {
        if ($_GET['artist'] == $artist) {
            echo "<option selected='selected' value='${artist}'>$artist</option>";
        } else {
            echo "<option value='${artist}'>$artist</option>";
        }
    }
    echo "</select>";
    if ($_GET['artist']) {
        echo "<form action='/melonbooks/index.php' id='remove_artist' method='post'>";
        echo "  <input type='submit' name='submit' value='Remove Artist'>";
        echo "</form>";
    }
	#echo "<a href='/melonbooks/index.php'>All</a></br>";
    #foreach ($artists as $artist) {
    #   echo "<a href='/melonbooks/index.php?artist=${artist}'>$artist</a></br>";
    #}
    ?>
    </div>

    <div>
      <?php
	 if ($_GET['artist']) {
	 $artist = $_GET['artist'];
	 //echo "showing products for ${artist}.";
         $products_query = $pdo->prepare("SELECT url, title, artist, site, imgUrl, dateAdded, availability FROM products WHERE availability in ('Available', 'Preorder') AND artist = (:artist) ORDER BY dateAdded DESC, CAST(SUBSTR(url, INSTR(url, 'product_id=') + 11) AS INTEGER) DESC", [PDO::ATTR_CURSOR => PDO::CURSOR_FWDONLY]);
         $products_query->execute(['artist' => $artist]);
         } else {
         $products_query = $pdo->prepare("SELECT url, title, artist, site, imgUrl, dateAdded, availability FROM products WHERE availability in ('Available', 'Preorder') ORDER BY dateAdded DESC, CAST(SUBSTR(url, INSTR(url, 'product_id=') + 11) AS INTEGER) DESC", [PDO::ATTR_CURSOR => PDO::CURSOR_FWDONLY]);
         $products_query->execute([]);
         }
	 $results = $products_query->fetchAll();
      
         echo "<table class='main-table-grid'><tr>";
         if (!$is_phone) {
	     $cols = 3;
         } else {
	     $cols = 2;
	 }
         //header
	 for ($i = 0; $i < $cols; $i+=1) {
             echo "<th class='image-table-header' style='width:" . (50/$cols) . "%'>Product</th>";
             echo "<th class='product-info-table-header' style='width:" . (50/$cols) . "%'>Info</th>";
             //echo "<th style='width:" . (10/$cols) . "%'>Availability</th>";
         }
	 echo "</tr>";
         //data  
	 $ind = 0;
         foreach ($results as $row)  {
	     if ($ind%$cols==0) {
                 echo "<tr class='grid-table-row'>";
	     }
             echo "<td class='image-table-cell'>";
             echo "  <img src='{$row[4]}&height=250' class='product-image'>";
             echo "</td>";
             echo "<td class='product-info-table-cell'>";
             //echo "<img style='max-width:30px; height:auto' src='https://melonbooks.co.jp/apple-touch-icon.png'><br>";
             echo "<a href={$row[0]}>{$row[1]}</a>";
             echo "<br>Artist: <a class='product-artist-name'>{$row[2]}</a>";
	     echo "<br>Date: {$row[5]}";
             //echo "</td>";
             if ($row[6] == "Available") {
	         $color = "#dd7722";
             } else if ($row[6] == "Preorder") {
                 $color = "#3322cc";
             } else if ($row[6] == "NotAvailable") {
                 $color = "#ff3333";
             } else {
                 $color = "black";
             }
             echo "<br><br>Status: <a style='color:{$color}'>{$row[6]}</a>";
	     echo "</td>";
             if ($ind%$cols==$cols-1) {
                 echo "</tr>";
	     }
	     $ind = $ind + 1;
         }
         if (count($results)%$cols!=0) {
	     echo "</tr>";
         }
	 echo "</table>";
         
      ?>
    </div>
</body>
</html>
